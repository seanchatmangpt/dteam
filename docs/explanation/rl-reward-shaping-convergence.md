# RL for Process Discovery: Reward Shaping and Convergence

*Diátaxis — Understanding-oriented explanation.*

---

## 1. Why Process Discovery is an Optimization Problem

Process discovery — inferring a Petri net model from an event log — is not a parsing problem. It would be simpler if it were. Parsing looks for the one structure consistent with a grammar. Discovery looks for a model that is maximally consistent with a log that may contain noise, exceptions, rare variants, and recording artifacts. There is rarely one right answer; there is a space of candidates ranging from the trivially sound (a net that accepts everything) to the trivially precise (a net that accepts only the traces it has seen), with the useful models somewhere in between.

This framing makes discovery an optimization problem: search the space of Petri net structures for the model that maximizes conformance to the training log, subject to soundness (the model must be a valid workflow net) and economy (the model should not be more complex than the evidence requires). The formal target is:

```
maximize fitness(net, log)
subject to is_structural_workflow_net(net)
         and verifies_state_equation_calculus(net)
minimizing mdl_score(net)
```

Heuristic approaches — the α-algorithm, region theory, inductive mining — solve specific constrained versions of this problem with known guarantees and known failure modes. They work well when the log is clean and the process is structured. When the log contains noise, when the process has optional paths, or when the discovery target must be tuned online as new traces arrive, heuristics break down in ways that are hard to diagnose.

Reinforcement learning offers a different angle. Instead of encoding process-structural assumptions into the discovery algorithm, RL lets the agent learn which structural modifications improve the score. It makes no assumption about the process shape. The exploration-exploitation trade-off means it will find better models than pure hill-climbing while avoiding the overfitting that memorization would cause. The score function is the constraint: if you design the reward to penalize unsound or overly complex models, the agent learns to avoid them.

---

## 2. Designing the State Space: What the Agent Sees

A reinforcement learning agent cannot operate on raw Petri net structures directly — the state space of all possible nets is effectively infinite, which makes tabular convergence impossible. The `RlState<const WORDS: usize>` struct in `src/lib.rs` is the solution: it compresses the agent's view of the world into a fixed-size, discrete, hashable structure:

```rust
pub struct RlState<const WORDS: usize> {
    pub health_level: i8,
    pub event_rate_q: i8,
    pub activity_count_q: i8,
    pub spc_alert_level: i8,
    pub drift_status: i8,
    pub rework_ratio_q: i8,
    pub circuit_state: i8,
    pub cycle_phase: i8,
    pub marking_mask: KBitSet<WORDS>,
    pub activities_hash: u64,
    pub ontology_mask: KBitSet<16>,
    pub universe: Option<Universe64>,
}
```

Every `i8` field is a quantized observation. `health_level` compresses the current model's overall fitness into one of 256 levels. `event_rate_q` quantizes the rate at which new event types are appearing. `activity_count_q` encodes how many distinct activities the model currently handles. The *q* suffix signals that these are not raw measurements — they are discretized bins.

Why quantization? Because tabular Q-learning requires a finite state space. The number of distinct `RlState` values is bounded: with eight `i8` fields and a 64-bit marking mask, the space is large but enumerable, and in practice the visited states during a training run form a small fraction of the theoretical maximum. This is what makes convergence achievable. A continuous state representation would require function approximation, which introduces approximation error and can diverge.

The `features()` method bridges the two worlds: it flattens the `RlState` fields into a `[f32; 16]` array, suitable for neural network or linear function approximators. Tabular Q-learning uses the hash of the struct directly; function approximation uses `features()`. The same state representation serves both regimes.

---

## 3. The Reward Decomposition

The reward function is the teacher, the examiner, and the editor simultaneously — the music analogy made arithmetic. A music teacher might say: "play it with feeling" (maximize fitness), "but keep the fingering clean" (structural soundness bonus), "and don't add unnecessary ornaments" (complexity penalty), "and the other students in the conservatory generally agree with your interpretation" (ensemble vote). Each of these is a separate criterion, and their relative weights encode a judgment about what matters most.

From `automation.rs` (lines 139–142):

```rust
// Reward = fitness + β·soundness − λ·complexity [+ γ·ensemble_vote]
let ensemble_bonus = ensemble_vote.unwrap_or(0.0) * 0.3;
let reward =
    avg_f as f32 + beta * (1.0 - unsoundness_u) - lambda * complexity_c + ensemble_bonus;
```

`avg_f` is the average token replay fitness across the training log — the primary signal. It ranges from 0.0 (the model explains none of the observed behavior) to 1.0 (the model explains all of it perfectly). This is the dominant term.

`beta * (1.0 - unsoundness_u)` adds a structural correctness bonus. `unsoundness_u` is `structural_unsoundness_score()`, which measures how far the net is from being a valid workflow net — whether it has disconnected transitions, multiple source places, or missing arcs. The weight β (typically 0.5–0.8) is calibrated so that a fully sound net receives a meaningful bonus, but not so large that the agent sacrifices fitness for soundness alone.

`lambda * complexity_c` is the Occam penalty. `complexity_c` is the sum of transition count and arc count. A larger model pays a larger penalty. The weight λ (typically 0.01–0.05) is kept small because the primary concern is fitness; complexity is a tiebreaker, not a primary objective. If λ were too large, the agent would converge to trivially simple nets that have no explanatory power.

`ensemble_vote * 0.3` is the cross-validation signal (γ = 0.3 hardcoded). When an ensemble of external classifiers agrees with the current model's labeling of traces, the reward rises. This term prevents the agent from overfitting to artifacts of a single training run — if five other classifiers disagree, the signal should be discounted.

---

## 4. MDL Penalty: Occam's Razor in Silicon

The Minimum Description Length (MDL) principle states that the best model for a dataset is the one that minimizes the total description length of the model plus the data given the model. A simple model requires few bits to describe itself; a complex model requires more. If a complex model only marginally improves the fit to the data, MDL prefers the simpler one.

The `mdl_score_with_ontology` implementation in `petri_net.rs`:

```rust
pub fn mdl_score_with_ontology(&self, ontology_size: Option<usize>) -> f64 {
    let t = self.transitions.len() as f64;
    let a = self.arcs.len() as f64;
    let vocabulary_size = ontology_size.map(|s| s as f64).unwrap_or(t);
    t + (a * vocabulary_size.log2())
}
```

This is MDL in the simplest useful form. The model requires `t` bits to specify which transitions exist (one bit per activity in the vocabulary that is included). Each arc requires `log₂(|V|)` bits to specify which transition it connects to, where `|V|` is the vocabulary size. The total MDL score is therefore the number of transitions plus the logarithmic encoding cost of all arcs.

Why is this information-theoretically principled? Because `log₂(|V|)` is the minimum number of bits needed to encode a choice among `|V|` options (Shannon's source coding theorem). A net with many arcs connecting many distinct transitions has high description complexity — it requires many bits to specify. A net that routes everything through a single sequence of activities has low description complexity. MDL quantifies this formally.

In practice, MDL serves a different role in the reward than in model selection. During training, `complexity_c` in the reward uses the simpler proxy `|T| + |A|`. The formal MDL score is used for final model selection among candidates that have already passed the convergence gate. The distinction matters: during training you want a smooth gradient signal; for final selection you want a principled comparison across structurally different nets.

---

## 5. Double Q-Learning and the Overestimation Problem

Standard Q-learning updates the value of a state-action pair using the maximum Q-value over the next state's actions:

```
Q(s, a) ← Q(s, a) + α[r + γ·max_a' Q(s', a') − Q(s, a)]
```

The `max` operator introduces a systematic bias. If Q-values are noisy estimates (as they always are early in training), `max_a' Q(s', a')` consistently selects the noisy overestimate. This over-optimism cascades: the agent converges to a policy that looks better on paper than it is in practice, and may keep exploring regions of the state space that appear promising but are not.

In process discovery, this problem is amplified by the sparsity of the state space. Most combinations of net structure and log statistics are never visited. The Q-values for unvisited states remain at their initial values (typically 0.0), and the agent must learn from the few states it does visit that the Q-value of an unvisited neighbor is not actually 0.0. With overestimation, the agent is systematically told that unexplored territory is better than it is.

Double Q-learning decouples action selection from action evaluation. The update in `double_q.rs` (lines 94–121) randomly assigns one of two tables (Q_A or Q_B) the role of selector and the other the role of evaluator:

```rust
if self.rng.borrow_mut().bool() {
    let next_vals = get_q_values::<S, A>(&*qa, &next_state);
    let best_next_idx = greedy_index(next_vals);
    let next_q = qb.get(h_next).map(|vals| vals[best_next_idx]).unwrap_or(0.0);
    // ... update qa
} else {
    // ... symmetric: qb selects, qa evaluates
}
```

Q_A selects which action looks best in the next state; Q_B evaluates how good that action actually is. Because the two tables are updated independently with different samples, they accumulate different noise. When both tables agree that an action is good, the estimate is more reliable. When they disagree, the overestimate is partially corrected by the cross-evaluation.

For process discovery specifically, Double Q-Learning matters because the `Optimize` and `Rework` actions can appear superficially rewarding (they change the model, which briefly changes the reward signal) before the new model settles and the reward is properly computed.

---

## 6. The Three-Condition Convergence Gate

Early stopping in `automation.rs` uses a conjunction of three independent conditions:

```rust
if avg_f >= config.discovery.fitness_stopping_threshold && is_sound && verifies_calculus {
    break;
}
```

This is the formal anti-lie statement. Each condition blocks a different form of gaming.

A net that accepts all traces has fitness 1.0 but is not a workflow net — `is_structural_workflow_net()` rejects it because it has no unique source or sink place, or has transitions with no input arcs. Fitness alone is gameable; soundness blocks the trivial case.

A net that is structurally sound but has no arcs connecting its transitions to places passes `is_structural_workflow_net()` for the same reason a disconnected graph might pass a naive connectivity check: structure without behavior. `verifies_state_equation_calculus()` closes this loophole by requiring that every transition in the net actually consumes from at least one place and produces into at least one place. It checks the incidence matrix column by column, verifying both `consumes = true` and `produces = true` for every transition.

The fitness threshold alone is gameable with a flower model (one place, all transitions are self-loops) or a parallel net (all activities run in parallel, accepting any permutation). The soundness requirement alone is gameable with an overfitting net that memorizes every training trace. The calculus requirement alone is gameable with a net that has the right structure on paper but routes tokens along paths that never appear in the log.

Only the conjunction of all three conditions is resistant to these gaming strategies simultaneously. A net that passes all three is saying: the observed traces fit the model well; the model has valid workflow structure; and every piece of that structure is causally connected to token flow. That is what convergence means.

---

## 7. SARSA, Expected SARSA, and REINFORCE: When On-Policy Matters

`dteam` implements four reinforcement learning algorithms: `QLearning`, `DoubleQLearning`, `SARSAAgent`, and `ExpectedSARSAAgent`, plus `ReinforceAgent` for policy gradient. Understanding why the production training loop in `automation.rs` uses `QLearning` (rather than the alternatives) requires understanding the trade-off.

Q-learning is off-policy: it learns the value of the optimal policy regardless of the exploration policy being followed. This means the agent can follow an epsilon-greedy exploration strategy (random actions with probability ε) while still learning what the best deterministic policy would be. Off-policy learning converges faster in sparse state spaces because every experience, including exploratory ones, contributes to the optimal value function.

SARSA is on-policy: it learns the value of the policy currently being followed, including its exploration component. SARSA is more conservative — it accounts for the fact that during execution the agent will sometimes take random actions. For process discovery, where the cost of a bad action (emitting a structurally unsound net) is absorbed and corrected in the next epoch, the extra conservatism of SARSA rarely pays off. The three-condition convergence gate provides the stability guarantee that SARSA's conservative updates would otherwise provide.

Expected SARSA improves SARSA by using the expected value over all actions (weighted by their probability under the current policy) rather than the value of the actually taken action. This reduces variance at the cost of slightly more computation. In very stochastic environments it outperforms both Q-learning and SARSA; in the low-action-count case (three actions: Idle, Optimize, Rework), the variance reduction is marginal.

REINFORCE is a policy gradient method, appropriate when the action space is continuous or when the optimal policy is stochastic. In process discovery, the actions are discrete and the optimal policy is deterministic (once we know the current fitness and soundness state, the right action is determined). REINFORCE would add approximation overhead without benefit.

---

## 8. The Minimal Action Surface: Idle, Optimize, Rework

The `RlAction` enum in `src/lib.rs` exposes exactly three actions:

```rust
pub enum RlAction {
    Idle,
    Optimize,
    Rework,
}
```

This is a deliberate design choice, not an incomplete implementation. The action surface is minimal by design.

More actions mean a larger Q-table: for a tabular agent with state count S and action count A, the Q-table has S × A entries. With three actions, each state requires three Q-values. With twenty actions (specific structural modifications), each state would require twenty Q-values, and the agent would need twenty times as many observations per state to reliably estimate all of them. In a sparse state space — as process discovery always is — this kills convergence.

More actions also mean more opportunity for the agent to find spurious reward signals. If the action set included specific transitions by label, the agent could learn correlations between activity names and rewards that do not generalize. Three abstract actions — do nothing, make the model better, fix structural problems — force the agent to learn at the right level of abstraction.

The mapping from abstract actions to concrete behavior lives in `DefaultKernel::propose()`, which translates `RlAction::Optimize` into one of several `AutonomicAction` variants (add a transition, add an arc, adjust a weight), selected based on the current state. The RL agent does not know about specific structural operations; it knows only the abstract intent. The `DefaultKernel` knows about structural operations but not about reward optimization. This separation keeps both components simpler and more testable.

The `Idle` action is equally important. Without it, the agent would be required to make structural changes every epoch, preventing convergence even after an excellent model is found. `Idle` allows the agent to stay at a good solution once found, which cooperates with the three-condition convergence gate: once all conditions are satisfied, `Idle` is the correct action, and the agent learns to take it.
