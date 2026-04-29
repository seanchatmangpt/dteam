//! POWL8 kinetic partial-order workflow primitive.
//!
//! Minimal subset adapted from `unibit-powl`. Supports up to [`MAX_NODES`]
//! nodes with a [`BinaryRelation`] bit-matrix expressing partial orders.
//!
//! The kinetic dialect schedules cognitive breed invocations (Activity nodes
//! reference [`crate::verdict::Breed`]) over a partial order. Plans are
//! validated by [`Powl8::shape_match`]: bounds, child indices, and acyclicity
//! (Kahn's algorithm on PartialOrder submatrices, plus DFS over
//! `OperatorSequence` dependencies).

use crate::verdict::{Breed, PlanAdmission};

/// Maximum number of nodes in a POWL8 plan, bounding `BinaryRelation` to a
/// 64×64 bit-matrix.
pub const MAX_NODES: usize = 64;

/// Bit-packed adjacency matrix for partial orders over up to [`MAX_NODES`]
/// nodes.
///
/// `words[src]` holds a bit at position `tgt` iff edge `src → tgt` is present.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinaryRelation {
    /// Per-source row of target bits: `words[src] & (1 << tgt) != 0` iff
    /// edge `src → tgt` exists.
    words: [u64; MAX_NODES],
}

impl BinaryRelation {
    /// Create an empty relation with no edges.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            words: [0u64; MAX_NODES],
        }
    }

    /// Add an edge from `src` to `tgt`. Out-of-bounds indices are ignored.
    pub fn add_edge(&mut self, src: usize, tgt: usize) {
        if src < MAX_NODES && tgt < MAX_NODES {
            self.words[src] |= 1u64 << tgt;
        }
    }

    /// Return `true` iff an edge from `src` to `tgt` is present.
    #[must_use]
    pub const fn is_edge(&self, src: usize, tgt: usize) -> bool {
        if src < MAX_NODES && tgt < MAX_NODES {
            (self.words[src] >> tgt) & 1 == 1
        } else {
            false
        }
    }
}

impl Default for BinaryRelation {
    fn default() -> Self {
        Self::new()
    }
}

/// Plan node: either a marker, an Activity invoking a [`Breed`], a sub-plan
/// with its own partial order, or a binary operator over child indices.
#[derive(Clone, Copy, Debug)]
pub enum Powl8Node {
    /// No-op placeholder; treated as advanced once entered.
    Silent,
    /// Single breed invocation.
    Activity(Breed),
    /// Sub-plan with explicit partial order over `count` children located at
    /// indices `start..start + count` in the parent plan's node array.
    PartialOrder {
        /// Index of the first child node in the plan.
        start: u16,
        /// Number of consecutive child nodes participating in the partial order.
        count: u16,
        /// Edges of the partial order, indexed locally from 0..count.
        rel: BinaryRelation,
    },
    /// Children execute in sequence — `b` follows `a`.
    OperatorSequence {
        /// Index of the predecessor child node.
        a: u16,
        /// Index of the successor child node.
        b: u16,
    },
    /// Children execute in parallel — `a` and `b` are independent.
    OperatorParallel {
        /// Index of the first parallel child.
        a: u16,
        /// Index of the second parallel child.
        b: u16,
    },
    /// Plan entry marker; treated as advanced from the outset.
    StartNode,
    /// Plan exit marker; reachable only after all predecessors are advanced.
    EndNode,
}

/// Kinetic partial-order workflow plan, bounded to [`MAX_NODES`] nodes.
#[derive(Clone, Debug)]
pub struct Powl8 {
    /// Plan node array; `nodes.len() <= MAX_NODES` is enforced by [`Powl8::push`].
    pub nodes: Vec<Powl8Node>,
    /// Index of the root entry point in `nodes`.
    pub root: u16,
}

impl Powl8 {
    /// Create an empty plan with `root = 0`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: 0,
        }
    }

    /// Append a node and return its index. Returns
    /// [`PlanAdmission::Malformed`] if the plan is at capacity.
    pub fn push(&mut self, node: Powl8Node) -> Result<u16, PlanAdmission> {
        if self.nodes.len() >= MAX_NODES {
            return Err(PlanAdmission::Malformed);
        }
        let idx = self.nodes.len() as u16;
        self.nodes.push(node);
        Ok(idx)
    }

    /// Validate plan structure: bounds, valid breeds, acyclic relations.
    ///
    /// Returns `Ok(())` if Sound, `Err(PlanAdmission::Cyclic)` on detected
    /// cycle, and `Err(PlanAdmission::Malformed)` on out-of-bounds children
    /// or self-references in operators.
    pub fn shape_match(&self) -> Result<(), PlanAdmission> {
        // Bounds.
        if self.nodes.len() > MAX_NODES {
            return Err(PlanAdmission::Malformed);
        }
        if self.nodes.is_empty() {
            // An empty plan with root=0 has no valid root.
            return Err(PlanAdmission::Malformed);
        }
        if (self.root as usize) >= self.nodes.len() {
            return Err(PlanAdmission::Malformed);
        }

        let n = self.nodes.len();

        // Per-node structural checks.
        for (idx, node) in self.nodes.iter().enumerate() {
            match *node {
                Powl8Node::Silent | Powl8Node::StartNode | Powl8Node::EndNode => {}
                // The Breed enum itself bounds the discriminant to 0..=6;
                // the match here is purely defensive and total.
                Powl8Node::Activity(_) => {}
                Powl8Node::PartialOrder { start, count, rel } => {
                    let s = start as usize;
                    let c = count as usize;
                    if s.saturating_add(c) > n {
                        return Err(PlanAdmission::Malformed);
                    }
                    if c == 0 {
                        // Empty partial order is structurally fine but trivial.
                        continue;
                    }
                    // Kahn's algorithm on the local count-by-count submatrix.
                    // Local index `i` corresponds to global index `start + i`.
                    let mut indegree = [0u16; MAX_NODES];
                    for i in 0..c {
                        for j in 0..c {
                            if i != j && rel.is_edge(i, j) {
                                indegree[j] = indegree[j].saturating_add(1);
                            }
                        }
                    }
                    let mut queue: [usize; MAX_NODES] = [0usize; MAX_NODES];
                    let mut qhead = 0usize;
                    let mut qtail = 0usize;
                    for i in 0..c {
                        if indegree[i] == 0 {
                            queue[qtail] = i;
                            qtail += 1;
                        }
                    }
                    let mut peeled = 0usize;
                    while qhead < qtail {
                        let v = queue[qhead];
                        qhead += 1;
                        peeled += 1;
                        for w in 0..c {
                            if v != w && rel.is_edge(v, w) {
                                indegree[w] = indegree[w].saturating_sub(1);
                                if indegree[w] == 0 {
                                    queue[qtail] = w;
                                    qtail += 1;
                                }
                            }
                        }
                    }
                    if peeled != c {
                        return Err(PlanAdmission::Cyclic);
                    }
                }
                Powl8Node::OperatorSequence { a, b }
                | Powl8Node::OperatorParallel { a, b } => {
                    let ai = a as usize;
                    let bi = b as usize;
                    if ai >= n || bi >= n {
                        return Err(PlanAdmission::Malformed);
                    }
                    if ai == idx || bi == idx {
                        return Err(PlanAdmission::Malformed);
                    }
                }
            }
        }

        // Whole-plan DFS-with-color over OperatorSequence dependencies.
        // Each OperatorSequence { a, b } contributes the edge a → b.
        // PartialOrder edges are handled per-node above; we still incorporate
        // them in the global cycle check by walking edges out of each
        // partial-order child to enforce composite acyclicity.
        const WHITE: u8 = 0;
        const GRAY: u8 = 1;
        const BLACK: u8 = 2;
        let mut color = [WHITE; MAX_NODES];

        // Build out-edges per node lazily inside dfs to avoid allocations.
        for start_node in 0..n {
            if color[start_node] != WHITE {
                continue;
            }
            // Iterative DFS using a small stack.
            let mut stack: [(usize, u32); MAX_NODES] = [(0usize, 0u32); MAX_NODES];
            let mut depth = 0usize;
            stack[depth] = (start_node, 0);
            depth += 1;
            color[start_node] = GRAY;
            while depth > 0 {
                let (u, edge_idx) = stack[depth - 1];
                if let Some(v) = self.nth_outgoing(u, edge_idx as usize) {
                    stack[depth - 1].1 = edge_idx + 1;
                    if v >= n {
                        return Err(PlanAdmission::Malformed);
                    }
                    match color[v] {
                        WHITE => {
                            color[v] = GRAY;
                            if depth >= MAX_NODES {
                                // Defensive: depth cannot exceed node count.
                                return Err(PlanAdmission::Malformed);
                            }
                            stack[depth] = (v, 0);
                            depth += 1;
                        }
                        GRAY => {
                            return Err(PlanAdmission::Cyclic);
                        }
                        _ => {}
                    }
                } else {
                    color[u] = BLACK;
                    depth -= 1;
                }
            }
        }

        Ok(())
    }

    /// Yield the `k`th outgoing edge target from node `u`, or `None` once
    /// exhausted. Outgoing edges are derived from `OperatorSequence` (a → b)
    /// and from `PartialOrder` local rel edges (mapped back to global indices).
    fn nth_outgoing(&self, u: usize, k: usize) -> Option<usize> {
        let mut counter = 0usize;
        // Outgoing edges from any node are produced by the structure of every
        // node in the plan, so we scan the plan once per query. This is O(n²)
        // overall for shape_match, which is fine at MAX_NODES = 64.
        for node in &self.nodes {
            match *node {
                Powl8Node::OperatorSequence { a, b } => {
                    if (a as usize) == u {
                        if counter == k {
                            return Some(b as usize);
                        }
                        counter += 1;
                    }
                }
                Powl8Node::PartialOrder { start, count, rel } => {
                    let s = start as usize;
                    let c = count as usize;
                    if u >= s && u < s + c {
                        let local_u = u - s;
                        for j in 0..c {
                            if local_u != j && rel.is_edge(local_u, j) {
                                if counter == k {
                                    return Some(s + j);
                                }
                                counter += 1;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Compute predecessors for every node into `preds[i]` using
    /// `OperatorSequence`, `OperatorParallel`, and `PartialOrder` edges.
    ///
    /// `OperatorSequence { a, b }` adds `a` as a predecessor of `b`.
    /// `OperatorParallel { a, b }` adds no predecessor edges between `a` and
    /// `b` (they are independent).
    /// `PartialOrder { start, count, rel }` adds local-edge predecessors
    /// mapped back to global indices.
    ///
    /// `preds[i]` is a 64-bit mask over node indices; bit `j` is set iff `j`
    /// is a direct predecessor of `i`.
    pub fn predecessor_masks(&self) -> [u64; MAX_NODES] {
        let mut preds = [0u64; MAX_NODES];
        let n = self.nodes.len().min(MAX_NODES);
        for node in &self.nodes {
            match *node {
                Powl8Node::OperatorSequence { a, b } => {
                    let ai = a as usize;
                    let bi = b as usize;
                    if ai < n && bi < n {
                        preds[bi] |= 1u64 << ai;
                    }
                }
                Powl8Node::PartialOrder { start, count, rel } => {
                    let s = start as usize;
                    let c = count as usize;
                    if s.saturating_add(c) > n {
                        continue;
                    }
                    for i in 0..c {
                        for j in 0..c {
                            if i != j && rel.is_edge(i, j) {
                                preds[s + j] |= 1u64 << (s + i);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        preds
    }
}

impl Default for Powl8 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_relation_set_and_query() {
        let mut r = BinaryRelation::new();
        r.add_edge(0, 1);
        r.add_edge(2, 3);
        assert!(r.is_edge(0, 1));
        assert!(r.is_edge(2, 3));
        assert!(!r.is_edge(1, 0));
        assert!(!r.is_edge(63, 63));
    }

    #[test]
    fn binary_relation_oob_ignored() {
        let mut r = BinaryRelation::new();
        r.add_edge(MAX_NODES, 0);
        r.add_edge(0, MAX_NODES);
        assert!(!r.is_edge(MAX_NODES, 0));
        assert!(!r.is_edge(0, MAX_NODES));
    }

    #[test]
    fn powl8_push_caps_at_max_nodes() {
        let mut p = Powl8::new();
        for _ in 0..MAX_NODES {
            p.push(Powl8Node::Silent).unwrap();
        }
        assert!(matches!(
            p.push(Powl8Node::Silent),
            Err(PlanAdmission::Malformed)
        ));
    }

    #[test]
    fn shape_match_empty_is_malformed() {
        let p = Powl8::new();
        assert!(matches!(p.shape_match(), Err(PlanAdmission::Malformed)));
    }

    #[test]
    fn shape_match_root_oob_is_malformed() {
        let mut p = Powl8::new();
        p.push(Powl8Node::StartNode).unwrap();
        p.root = 5;
        assert!(matches!(p.shape_match(), Err(PlanAdmission::Malformed)));
    }

    #[test]
    fn shape_match_acyclic_partial_order_is_sound() {
        let mut p = Powl8::new();
        let s = p.push(Powl8Node::StartNode).unwrap();
        p.root = s;
        let _e = p.push(Powl8Node::Activity(Breed::Eliza)).unwrap();
        let _m = p.push(Powl8Node::Activity(Breed::Mycin)).unwrap();
        let _x = p.push(Powl8Node::Activity(Breed::Strips)).unwrap();
        let mut rel = BinaryRelation::new();
        rel.add_edge(0, 1);
        rel.add_edge(1, 2);
        let _po = p.push(Powl8Node::PartialOrder {
            start: 1,
            count: 3,
            rel,
        }).unwrap();
        assert!(p.shape_match().is_ok());
    }

    #[test]
    fn shape_match_cyclic_partial_order_is_cyclic() {
        let mut p = Powl8::new();
        p.push(Powl8Node::StartNode).unwrap();
        p.push(Powl8Node::Activity(Breed::Eliza)).unwrap();
        p.push(Powl8Node::Activity(Breed::Mycin)).unwrap();
        p.push(Powl8Node::Activity(Breed::Strips)).unwrap();
        let mut rel = BinaryRelation::new();
        rel.add_edge(0, 1);
        rel.add_edge(1, 2);
        rel.add_edge(2, 0);
        p.push(Powl8Node::PartialOrder {
            start: 1,
            count: 3,
            rel,
        }).unwrap();
        assert!(matches!(p.shape_match(), Err(PlanAdmission::Cyclic)));
    }

    #[test]
    fn shape_match_self_referencing_operator_is_malformed() {
        let mut p = Powl8::new();
        p.push(Powl8Node::StartNode).unwrap();
        p.push(Powl8Node::Activity(Breed::Eliza)).unwrap();
        p.push(Powl8Node::OperatorSequence { a: 1, b: 2 }).unwrap();
        // Index 2 references itself via b=2.
        assert!(matches!(p.shape_match(), Err(PlanAdmission::Malformed)));
    }

    #[test]
    fn shape_match_seq_oob_child_is_malformed() {
        let mut p = Powl8::new();
        p.push(Powl8Node::StartNode).unwrap();
        p.push(Powl8Node::OperatorSequence { a: 0, b: 99 }).unwrap();
        assert!(matches!(p.shape_match(), Err(PlanAdmission::Malformed)));
    }

    #[test]
    fn shape_match_seq_cycle_detected_globally() {
        let mut p = Powl8::new();
        // 4 nodes; build a cycle 0 -> 1 -> 2 -> 0 via OperatorSequence rows.
        p.push(Powl8Node::Activity(Breed::Eliza)).unwrap();
        p.push(Powl8Node::Activity(Breed::Mycin)).unwrap();
        p.push(Powl8Node::Activity(Breed::Strips)).unwrap();
        p.push(Powl8Node::OperatorSequence { a: 0, b: 1 }).unwrap();
        p.push(Powl8Node::OperatorSequence { a: 1, b: 2 }).unwrap();
        p.push(Powl8Node::OperatorSequence { a: 2, b: 0 }).unwrap();
        assert!(matches!(p.shape_match(), Err(PlanAdmission::Cyclic)));
    }
}
