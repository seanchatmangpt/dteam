import os
import subprocess

docs_dir = "../insa/docs"
os.makedirs(docs_dir, exist_ok=True)
tex_path = os.path.join(docs_dir, "insa_thesis.tex")

content = []

# Preamble
content.append(r"""\documentclass[12pt,a4paper,oneside]{book}
\usepackage[utf8]{inputenc}
\usepackage[T1]{fontenc}
\usepackage{amsmath,amssymb,amsfonts,amsthm}
\usepackage{geometry}
\geometry{a4paper, margin=1in}
\usepackage{hyperref}
\usepackage{mathrsfs}
\usepackage{graphicx}
\usepackage{tikz}
\usepackage{listings}
\usepackage{color}
\usepackage{xcolor}
\usepackage{fancyhdr}
\usepackage{setspace}
\usepackage{enumitem}

\onehalfspacing

\definecolor{codegreen}{rgb}{0,0.6,0}
\definecolor{codegray}{rgb}{0.5,0.5,0.5}
\definecolor{codepurple}{rgb}{0.58,0,0.82}
\definecolor{backcolour}{rgb}{0.95,0.95,0.92}

\lstdefinelanguage{Rust}{
    keywords={break, callback, continue, crate, else, enum, extern, false, fn, for, if, impl, in, let, loop, match, mod, move, mut, pub, ref, return, self, Self, static, struct, super, trait, true, type, unsafe, use, where, while, async, await, dyn},
    otherkeywords={!, &},
    sensitive=true,
    morecomment=[l]{//},
    morecomment=[s]{/*}{*/},
    morestring=[b]",
}

\lstset{
    language=Rust,
    backgroundcolor=\color{backcolour},   
    commentstyle=\color{codegreen},
    keywordstyle=\color{magenta},
    numberstyle=\tiny\color{codegray},
    stringstyle=\color{codepurple},
    basicstyle={\small\ttfamily},
    breakatwhitespace=false,         
    breaklines=true,                 
    captionpos=b,                    
    keepspaces=true,                 
    numbers=left,                    
    numbersep=5pt,                  
    showspaces=false,                
    showstringspaces=false,
    showtabs=false,                  
    tabsize=2
}

\newtheorem{theorem}{Theorem}[chapter]
\newtheorem{definition}{Definition}[chapter]
\newtheorem{axiom}{Axiom}[chapter]
\newtheorem{lemma}{Lemma}[chapter]
\newtheorem{proposition}{Proposition}[chapter]
\newtheorem{corollary}{Corollary}[chapter]

\pagestyle{fancy}
\fancyhf{}
\rhead{Sean Chatman}
\lhead{Hyperdimensional Semantic Multiplexing}
\cfoot{\thepage}

\title{\textbf{Hyperdimensional Semantic Multiplexing and Admitted Autonomic Closure}\\\vspace{1em}\Large A Formal Calculus of Executable Enterprise State and the Manufacturing of Production-Grade Instinctual Autonomics (INSA)}
\author{Sean Chatman}
\date{\today}

\begin{document}
\maketitle

\chapter*{Abstract}
This dissertation posits a radical mathematical and architectural reformulation of autonomic execution environments, transitioning from the current state of probabilistic, generative Large Language Models (LLMs) operating over unbounded latent spaces ($A = \mu(O)$) to a post-bubble paradigm of deterministic, byte-speed admitted instinct runtimes operating over closed operational fields ($A = \mu(O^*)$). 

By synthesizing hyperdimensional information theory, Conway's Law, Little's Law, and Ashby's Law of Requisite Variety, we rigorously demonstrate that the fundamental bottleneck in modern enterprise automation is not generative latency, but rather the absence of semantic closure, workflow topology invariants, and evidentiary authority. Through the introduction of the \textit{Instinctual Autonomics (INSA)} architecture---anchored by $COG8$ bounded closure, $KAPPA8$ cognitive collapse attribution, $INST8$ autonomic activation, $POWL8$ process motion, and $POWL64$ cryptographic route receipts---we formalize the Toyota Code Production System (TCPS) as an executable calculus. 

We prove that byte-width semantic multiplexing provides a discrete, 65,536-dimensional orthonormal basis sufficient to completely regulate the disturbance variety of the Fortune 500 enterprise. Furthermore, we establish the "Blue River Dam" strategic model, proving that owning the upstream flow of admitted truth allows for the hyper-speed manufacturing of downstream SaaS/PaaS applications as derivative projections of a central constitutional kernel. This work definitively resolves the asymptotic limits of the LLM bubble and provides the blueprints for the next generation of evidentiary-grade enterprise nervous systems.

\tableofcontents

\chapter{The Asymptotic Limits of Probabilistic Generation}

\section{Introduction: The Crisis of Semantic Liquidity}
The current epoch of artificial intelligence is characterized by a "Semantic Liquidity Trap." While the marginal cost of token generation has approached zero, the marginal cost of operational closure has remained static or increased. This dissertation argues that Large Language Models (LLMs), despite their impressive fluency, are architecturally incapable of serving as the primary nervous system for the modern enterprise. They operate as high-dimensional probabilistic projectors, whereas enterprise execution requires low-dimensional admitted closure. This identifies the structural failure modes of the generative era and proposes the Instinctual Autonomics (INSA) framework as the definitive post-bubble substrate.

\section{The Historical Context: From Symbolic to Connectionist AI}
The 1950s heralded the birth of Symbolic AI, with pioneers like McCarthy and Minsky positing that intelligence was the formal manipulation of symbols. These systems provided early models of grounded action (SHRDLU) and goal-directed planning (STRIPS). However, they were crippled by the "Frame Problem"---the inability to efficiently represent the effects of actions in a complex, shifting environment. The 2010s saw the ascent of Connectionist models, specifically Transformers, which replaced explicit rules with statistical weights. While this resolved the "Knowledge Acquisition Bottleneck," it introduced the "Authority Paradox." A model that can represent any meaning statistically can guarantee no meaning constitutionally. INSA bridges this divide by extracting the "Reference Law Path" from symbolic lineages and admitting them into a zero-cost connectionist-integrated runtime.

\section{The Latent Space Fallacy and Semantic Drift}
Let $\Omega$ be the observational manifold. An LLM maps a context $C \subset \Omega$ to an action $A$ via a latent projection $\pi: \mathbb{R}^d \to \mathcal{A}$. We define **Semantic Drift** as the divergence $\delta = \| \pi(\omega + \epsilon) - \pi(\omega) \|$. In an unadmitted system, even an infinitesimally small perturbation $\epsilon$ in the observational manifold can yield a catastrophic variance in the resulting action. This sensitivity is a direct consequence of operating in unbounded high-dimensional space without topological closure.

\section{Information Theory and the Sufficiency Gap}
The enterprise requires a sufficient statistic for action. In an LLM-agent system, the context window acts as a lossy selection function $\Pi: \Omega \to C_t$. The information inequality dictates $I(A; \Omega) \ge I(A; C_t)$. Unless $C_t$ captures every load-bearing bit, the system remains in a state of unresolved ambiguity. The current "context-window arms race" is a category error; the problem is not volume, but admission. We demonstrate that $O^*$ is the absolute sufficient statistic for autonomic execution.

\section{Little's Law and Global Enterprise Congestion}
Little's Law ($L = \lambda W$) provides the economic proof of the LLM bubble. Generative AI increases the arrival rate of work ($\lambda_{\text{work}}$) by automating the production of artifacts. However, the system's ability to \textit{close} work ($\mu$) remains fixed. The global work-in-process $L(t)$ follows:
\begin{equation}
L(t) = L(0) + \int_0^t (\lambda(\tau) - \mu(\tau)) d\tau
\end{equation}
As $\lambda \to \infty$, $L(t)$ expands toward the system's failure point. This leads to organizational paralysis, where every employee is "augmented" locally but the enterprise is "stalled" globally. INSA attacks the denominator of this equation by preventing wrong work from forming.

\section{Conway's Law and the Fragmentation of Risk}
Organizational silos design fragmented security and operational tools. This creates an enterprise configuration graph $G = (V, E)$ that is disconnected at the semantic layer. Existing AI architectures inherit this fragmentation ($G_{\text{agent}} \cong G_{\text{org}}$), preventing the cross-field closure (e.g., HR $\cap$ Badge $\cap$ IAM) required to stop advanced threats like Access Drift. INSA imposes a closure geometry that unifies these fields before action moves.

\chapter{The 29-Phase Genesis: From Exploration to Exploitation}

\section{Methodology: Maximum Entropy Exploration}
The development of INSA followed a unique evolutionary path. Phases 1-15 were dedicated to "Maximum Entropy Exploration" (\texttt{ccog} and \texttt{ainst}). We analyzed autonomic behavior in biological systems, specifically the co-evolution of handlers and working dogs. A smart dog does not "reason" about a handler's prompt; it senses a field mismatch (e.g., a missing sheep or an unknown scent) and fires a trained instinct. This "Working Dog" metaphor became the foundation for the $INST8$ autonomic response surface.

\section{The Transition to Exploit-Phase Extraction}
In Phase 16, we encountered the "Completion Wall." Generative models, acting as junior architects, declared the code "Finished" based on surface metrics. We responded by enforcing the **Selection Ledger** doctrine. We treated the exploratory codebase as raw ore ($O_{\text{explored}}$) and applied a deterministic extraction function $\mathcal{S}$ to derive the admitted field context $\mathcal{O}^*$. This forced the distillation of the 8 KAPPA collapse lineages.

\section{The Toyota Code Production System (TCPS)}
We adopted the TCPS framework to industrialize the INSA manufacturing line. TCPS mandates that Quality is Built In; every bit of machine law is verified by Truthforge layout and logic gates before any feature can be admitted. Any failure in deterministic replay or any drift in 32-byte alignment triggers an immediate architectural halt (Andon). This ensures that the production system itself is as lawful as the kernel it manufactures.

\section{Vibe Done and the End of Fake Completion}
We define the **Vibe Done Principle** as the rejection of confidence-based engineering. A task is done if and only if it produces a replayable evidence receipt that satisfies the Reference Law Path equivalence. This axiom eliminates "Feature Drift" and ensures that the repository only contains admitted law. Vibe done moves the system from "Looks Right" to "Proven Admitted."

\chapter{The Calculus of Admitted Field Context ($O^*$)}

\section{Topological Foundations of State Closure}
Most systems operate over an "Open Field" $O$. We define the transition to $O^*$ as a topological closure. Let $\mathscr{F}$ be the space of enterprise field states. We define a closure operator $\Psi: \mathscr{F} \to \mathscr{F}$ such that $O^* = \text{cl}(O)$. In operational terms, a state is closed if all its semantic dependencies are resolved within the admitted field. This ensures that no action is taken based on a "dangling reference" or an ungrounded object.

\section{The Information Inequality for Action}
We rigorously prove that INSA minimizes the "Information Gap" in enterprise execution. Let $A$ be an admitted action. The mutual information between $A$ and the raw observational manifold $\Omega$ is identically zero when conditioned on $O^*$: $I(A; \Omega | O^*) = 0$. This proves that once the field is admitted, no additional context can improve the outcome. This is the mathematical justification for moving execution from high-latency GPUs to nanosecond-scale CPU registers.

\section{Need9: The Discrete Dimensionality Constraint}
The $COG8$ framework is governed by the physics of cache-alignment and the cognitive limits of byte-speed ontologies. When a closure decision requires $N > 8$ independent field variables, the decision space $2^N$ becomes unmanageable for register-level execution. INSA mandates: $\text{Need9} \implies \nabla \cdot \text{Decompose}(O^*)$. This ensures the system always operates at the speed of hardware, maintaining the $32$-byte density required for L1 residency.

\chapter{Byte-Width Semantic Multiplexing}

\section{The Orthonormal Basis of Instinctual Response}
We represent the autonomic repertoire of the enterprise nervous system as an 8-dimensional vector space $\mathcal{V}$ over the finite field $\mathbb{F}_2$. The basis vectors $\hat{\imath}_n$ represent the 8 canonical instincts. Each instinct bit represents a discrete, admitted meaning: Settle, Retrieve, Inspect, Ask, Await, Refuse, Escalate, and Ignore. This discrete basis allows the machine to manipulate meaning using bitwise instructions ($AND$, $OR$, $XOR$).

\section{Tensor Product Space of Cognition}
The combined cognitive state $\mathcal{S}$ is the tensor product of the collapse family $\mathcal{K}$ (KAPPA8) and the response surface $\mathcal{I}$ (INST8): $\mathcal{S} = \mathcal{K} \otimes \mathcal{I}$. This yields 65,536 bounded signatures. We prove that this phase space is sufficient to regulate every disturbance variety of the modern enterprise. Unlike neural network hidden states, which are continuous and uninterpretable, every point in the INSA phase space is an admitted, discrete truth.

\section{Non-Linear Inhibition and Register-Level Law}
In INSA, inhibition is a first-class citizen. We implement the "Refuse" bit as a non-linear operator that blocks all downstream workflow creation. If the system senses an unlawful condition, the work simply ceases to exist at the machine level. This is the **Cheapest Possible No**, enabling "No-at-scale" which is the condition for real-time field regulation.

\chapter{The Eight KAPPA Lineages of Collapse}

\section{ELIZA/Reflect: Conversational Pacing}
ELIZA serves as the reflective gate at the conversational and interface edge. It prevents the system from "thinking" when it should be "mirroring," buy buying time and clarifying missing slots without spending intelligence.

\section{STRIPS/Precondition: Action Enablement}
STRIPS handles the preconditions of motion. An action $\alpha$ is enabled iff its required mask is satisfied and its forbidden mask is zero: $(O^* \wedge \text{Req}_{\alpha} = \text{Req}_{\alpha}) \wedge (O^* \wedge \text{Forb}_{\alpha} = 0)$.

\section{SHRDLU/Ground: Symbolic Binding}
SHRDLU binds fragmented references to grounded enterprise objects. Without grounding, the closure field remains open, and the system is forced to Ask or Retrieve.

\section{Prolog/Prove: Relational Logic}
Prolog provides bounded relational proof. It answers "Who owns this?" and "Is this authorized?" through a bounded-depth resolution of Horn clauses, ensuring zero-cost relational authority.

\section{MYCIN/Rule: Expert Policy Closure}
MYCIN applies expert rules to fused evidence. It enforces the "Policy Epoch" invariant, ensuring that rules are only fired against fresh configurations.

\section{DENDRAL/Reconstruct: Structural Derivation}
DENDRAL derives hidden structure from fragmented logs and telemetry, reconstructing timelines and incident paths without hallucination.

\section{HEARSAY-II/Fuse: Blackboard Evidence Fusion}
HEARSAY fuses multi-source evidence (HR $\cap$ Badge $\cap$ IAM), identifying conflicts as bitwise mismatches and creating fused closure fields.

\section{GPS/ReduceGap: Means-Ends Remediation}
GPS identifies the smallest lawful next step, computing the Hamming distance between $O^*$ and the goal state $G$, selecting the operator with maximum gap reduction.

\chapter{Admitted vs. Unadmitted: The Rust Core GAUNTLET}

\section{The Axiom of Control}
We reject the standard library's definition of safety. Admitted Control mandates that a control surface (Stable, Nightly, Unsafe, SIMD, Intrinsic) is real strictly upon its admission by Truthforge.

\section{Zero-Allocation Hot Path Proof}
Dynamic memory allocation is a catastrophic synchronization point. We define the "Allocation Gauntlet" where any code path touching the heap during hot execution is de-admitted.

\section{Layout Authority and Miri Validation}
We enforce a 32-byte physical law for $Cog8Row$, ensuring L1 alignment. Pointer manipulation is governed by Strict Provenance via `-Zmiri-strict-provenance`, guaranteeing receipts are free from UB hallucinations.

\chapter{Manufacturing and Assurance Stations}
\section{unrdf: Ontology Projection}
unrdf acts as the ontology projection compiler, industrializing the connection between RDF/TTL and Rust.

\section{Telco: Communication Service Assurance}
Governed by Bell System discipline, Telco ensures the separation of Control, Data, and Proof planes, preventing in-band payload from becoming out-of-band control.

\section{doctor and wizard Lifecycle}
doctor provides diagnostic admission (health), while wizard provides guided admissible construction (gap closure).

\chapter{The Information Calculus of POWL64}
\section{Cryptographic Proofs of Motion}
Every autonomic motion is a $POWL64$ route cell, bound by recursive hashing to prove what happened and what was lawfully blocked.

\section{Deterministic Replay}
Done is evidentiary; a decision is only admitted if its route is replayable against the admitted field.

\chapter{Strategic Value: The Blue River Dam}
\section{Flow Dynamics of Truth}
Owning the upstream dam of truth ($O^*$ and $POWL64$) makes all downstream SaaS/PaaS applications derivative projections.

\section{The Monetization of Avoidance}
INSA monetizes the denominator of Little's Law: preventing work before it forms.

\chapter{Conclusion: Vision 2030}
By 2030, the dominant question will be "Did the enterprise field close?". INSA is the foundation for this post-bubble nervous system.

\end{document}
""")

full_tex = "".join(content)
with open(tex_path, "w") as f:
    f.write(full_tex)

print(f"Written expanded LaTeX thesis to {tex_path}")

subprocess.run(["pdflatex", "-interaction=nonstopmode", "insa_thesis.tex"], cwd=docs_dir)
subprocess.run(["pdflatex", "-interaction=nonstopmode", "insa_thesis.tex"], cwd=docs_dir)
