#[cfg(test)]
mod tests {
    use super::super::ralph_optimizer::RalphOptimizer;

    #[test]
    fn test_branchless_optimization_path() {
        let target = 0xABCDE;
        let mut opt = RalphOptimizer::new(target);

        // Case 1: Match
        let action = opt.optimize_topology(target);
        assert!(action.is_none());

        // Case 2: Mismatch
        let action = opt.optimize_topology(0x12345);
        assert!(action.is_some());
        assert_eq!(action.unwrap().parameters, "DDS-AUTO: Topology Optimization");
    }
}
