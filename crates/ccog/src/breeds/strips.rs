//! STRIPS breed: transition admissibility via direct triple-pattern checks.

use anyhow::Result;
use oxigraph::model::NamedNode;
use crate::field::FieldContext;
use crate::graph::GraphIri;
use crate::powl::{Powl8, Powl8Node, MAX_NODES};
use crate::verdict::{Breed, PlanAdmission, PlanVerdict, TransitionVerdict};

/// STRIPS: Transition admissibility checking.
/// Evaluates whether a candidate operation can lawfully move the field from one graph state to another.
/// Preconditions: each `schema:DigitalDocument` must have a `prov:value`.

/// Check whether the candidate operation's transition is admissible.
/// Direct triple-pattern walk — no SPARQL parsing.
pub fn check_transition(
    candidate_iri: &GraphIri,
    field: &FieldContext,
) -> Result<TransitionVerdict> {
    let digital_document = NamedNode::new("https://schema.org/DigitalDocument")?;
    let prov_value = NamedNode::new("http://www.w3.org/ns/prov#value")?;

    let docs = field.graph.instances_of(&digital_document)?;
    for doc in &docs {
        if !field.graph.has_value_for(doc, &prov_value)? {
            return Ok(TransitionVerdict {
                admissible: false,
                blocked_by: vec![candidate_iri.clone()],
            });
        }
    }
    Ok(TransitionVerdict {
        admissible: true,
        blocked_by: Vec::new(),
    })
}

/// Admit a POWL8 plan against the field state.
///
/// Performs `shape_match` first. If the plan is not Sound (Cyclic or
/// Malformed), returns a [`PlanVerdict`] with empty `ready`/`blocked` and the
/// soundness reason. Otherwise computes per-node "advanced" status from the
/// field and classifies each node as ready or blocked based on its
/// predecessors.
///
/// Advanced semantics:
/// - `StartNode`, `Silent`: advanced = `true`.
/// - `EndNode`: advanced = `false` (entered only after all predecessors).
/// - `Activity(Strips)`: probed via [`check_transition`] against a sentinel
///   IRI. Advanced iff the field admits a STRIPS transition right now.
/// - `Activity(_)` for other breeds: defaults to `false` (placeholder; per-
///   breed kinetic probes will be added in later tracks).
/// - `OperatorSequence`, `OperatorParallel`, `PartialOrder`: not directly
///   advanced themselves; their children's advancement determines downstream
///   readiness.
pub fn admit_powl8(plan: &Powl8, field: &FieldContext) -> Result<PlanVerdict> {
    if let Err(admission) = plan.shape_match() {
        return Ok(PlanVerdict {
            ready: Vec::new(),
            blocked: Vec::new(),
            admissible: false,
            admission,
        });
    }

    let mut advanced = [false; MAX_NODES];
    let strips_probe_iri = GraphIri::from_iri("urn:ccog:powl8:strips-probe")?;
    for (idx, node) in plan.nodes.iter().enumerate() {
        if idx >= MAX_NODES {
            break;
        }
        advanced[idx] = match *node {
            Powl8Node::StartNode | Powl8Node::Silent => true,
            Powl8Node::EndNode => false,
            Powl8Node::Activity(Breed::Strips) => {
                check_transition(&strips_probe_iri, field)?.admissible
            }
            // Other breeds: not yet kinetically probed in this track.
            Powl8Node::Activity(_) => false,
            // Operators/sub-plans are structural; their advancement comes
            // from their constituent children, not from the operator node.
            Powl8Node::OperatorSequence { .. }
            | Powl8Node::OperatorParallel { .. }
            | Powl8Node::PartialOrder { .. } => false,
        };
    }

    admit_powl8_with_advanced(plan, &advanced)
}

/// Test-friendly variant of [`admit_powl8`] that accepts an explicit per-node
/// advanced bitmask instead of probing the field.
///
/// Classifies each node:
/// - **Ready** iff the node is not yet advanced *and* every direct
///   predecessor (per [`Powl8::predecessor_masks`]) is advanced.
/// - **Blocked** iff the node is not yet advanced *and* at least one direct
///   predecessor is not advanced.
/// - Already-advanced nodes appear in neither list.
///
/// `admissible` is `true` iff the plan is Sound and `ready` is non-empty
/// (or every node is already advanced — i.e., nothing remains to do).
pub fn admit_powl8_with_advanced(
    plan: &Powl8,
    advanced: &[bool; MAX_NODES],
) -> Result<PlanVerdict> {
    if let Err(admission) = plan.shape_match() {
        return Ok(PlanVerdict {
            ready: Vec::new(),
            blocked: Vec::new(),
            admissible: false,
            admission,
        });
    }

    let preds = plan.predecessor_masks();
    let n = plan.nodes.len();
    let mut ready: Vec<usize> = Vec::new();
    let mut blocked: Vec<usize> = Vec::new();

    for i in 0..n {
        if advanced[i] {
            continue;
        }
        // Skip pure structural operators/partial-order containers — they are
        // not themselves "ready/blocked" runtime activities.
        if matches!(
            plan.nodes[i],
            Powl8Node::OperatorSequence { .. }
                | Powl8Node::OperatorParallel { .. }
                | Powl8Node::PartialOrder { .. }
        ) {
            continue;
        }

        let p = preds[i];
        let mut all_preds_advanced = true;
        let mut bits = p;
        while bits != 0 {
            let j = bits.trailing_zeros() as usize;
            bits &= bits - 1;
            if j >= MAX_NODES || !advanced[j] {
                all_preds_advanced = false;
                break;
            }
        }
        if all_preds_advanced {
            ready.push(i);
        } else {
            blocked.push(i);
        }
    }

    let admissible = !ready.is_empty()
        || (0..n).all(|i| {
            advanced[i]
                || matches!(
                    plan.nodes[i],
                    Powl8Node::OperatorSequence { .. }
                        | Powl8Node::OperatorParallel { .. }
                        | Powl8Node::PartialOrder { .. }
                )
        });

    Ok(PlanVerdict {
        ready,
        blocked,
        admissible,
        admission: PlanAdmission::Sound,
    })
}
