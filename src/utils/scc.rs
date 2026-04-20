use crate::utils::dense_kernel::KBitSet;

/// Computes all SCCs of a generic K-Tier graph and returns them as an array of bitmasks.
/// Implementation based on transitive closure intersections.
#[allow(clippy::needless_range_loop)]
pub fn compute_sccs_generic<const WORDS: usize>(adj: &[KBitSet<WORDS>]) -> Vec<KBitSet<WORDS>> {
    let max_nodes = WORDS * 64;
    let mut sccs = Vec::new();
    let mut visited = KBitSet::<WORDS>::zero();

    let mut r = adj.to_vec();
    // Transitive Closure (Branchless)
    for k in 0..max_nodes {
        for i in 0..max_nodes {
            if r[i].contains(k) {
                let k_mask = r[k];
                r[i] = r[i].bitwise_or(k_mask);
            }
        }
    }

    // Transpose Reachability to get Column masks
    let mut rt = vec![KBitSet::<WORDS>::zero(); max_nodes];
    for i in 0..max_nodes {
        for j in 0..max_nodes {
            if r[i].contains(j) {
                let _ = rt[j].set(i);
            }
        }
    }

    for i in 0..max_nodes {
        if !visited.contains(i) {
            // SCC for node i is nodes reachable from i AND nodes that can reach i
            let mut scc = r[i].bitwise_and(rt[i]);
            let _ = scc.set(i);
            sccs.push(scc);
            visited = visited.bitwise_or(scc);
        }
    }

    sccs
}

/// A truly branchless version of compute_sccs using mask calculus.
#[allow(clippy::needless_range_loop)]
pub fn compute_sccs_branchless<const WORDS: usize>(adj: &[KBitSet<WORDS>]) -> Vec<KBitSet<WORDS>> {
    let max_nodes = WORDS * 64;
    let mut sccs = Vec::new();
    let mut visited = KBitSet::<WORDS>::zero();

    let mut r = adj.to_vec();

    // 1. Transitive Closure (Truly Branchless)
    for k in 0..max_nodes {
        let k_mask = r[k];
        for i in 0..max_nodes {
            // bit = r[i] contains k
            let bit = (r[i].words[k >> 6] >> (k & 63)) & 1;
            let mask = bit.wrapping_neg();
            for w in 0..WORDS {
                r[i].words[w] |= k_mask.words[w] & mask;
            }
        }
    }

    // 2. Transpose Reachability (Branchless)
    let mut rt = vec![KBitSet::<WORDS>::zero(); max_nodes];
    for i in 0..max_nodes {
        for j in 0..max_nodes {
            let bit = (r[i].words[j >> 6] >> (j & 63)) & 1;
            rt[j].words[i >> 6] |= bit << (i & 63);
        }
    }

    // 3. Extraction
    for i in 0..max_nodes {
        if !visited.contains(i) {
            let mut scc = r[i].bitwise_and(rt[i]);
            // Ensure self-reachability for SCC definition consistency
            let _ = scc.set(i);
            sccs.push(scc);
            visited = visited.bitwise_or(scc);
        }
    }

    sccs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scc_branchless_parity() {
        let mut adj = vec![KBitSet::<1>::zero(); 64];
        // Create a simple cycle: 0 -> 1 -> 2 -> 0
        let _ = adj[0].set(1);
        let _ = adj[1].set(2);
        let _ = adj[2].set(0);

        let sccs_gen = compute_sccs_generic(&adj);
        let sccs_br = compute_sccs_branchless(&adj);

        assert_eq!(sccs_gen.len(), sccs_br.len());
        for (a, b) in sccs_gen.iter().zip(sccs_br.iter()) {
            assert_eq!(a, b);
        }
    }
}
