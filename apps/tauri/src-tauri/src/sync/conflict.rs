use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorClock { pub clocks: HashMap<String, i64> }

impl VectorClock {
    pub fn new() -> Self { Self { clocks: HashMap::new() } }
    pub fn increment(&mut self, node_id: &str) { *self.clocks.entry(node_id.to_string()).or_insert(0) += 1; }
    pub fn merge(&mut self, other: &VectorClock) {
        for (node, count) in &other.clocks {
            let entry = self.clocks.entry(node.clone()).or_insert(0);
            if *count > *entry { *entry = *count; }
        }
    }
    pub fn get(&self, node_id: &str) -> i64 {
        *self.clocks.get(node_id).unwrap_or(&0)
    }
}

impl Default for VectorClock {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clock_is_empty() {
        let clock = VectorClock::new();
        assert!(clock.clocks.is_empty());
    }

    #[test]
    fn increment_single_node() {
        let mut clock = VectorClock::new();
        clock.increment("node-a");
        assert_eq!(clock.get("node-a"), 1);
    }

    #[test]
    fn increment_multiple_times() {
        let mut clock = VectorClock::new();
        clock.increment("node-a");
        clock.increment("node-a");
        clock.increment("node-a");
        assert_eq!(clock.get("node-a"), 3);
    }

    #[test]
    fn multiple_nodes_independent() {
        let mut clock = VectorClock::new();
        clock.increment("node-a");
        clock.increment("node-a");
        clock.increment("node-b");
        assert_eq!(clock.get("node-a"), 2);
        assert_eq!(clock.get("node-b"), 1);
    }

    #[test]
    fn get_unknown_node_returns_zero() {
        let clock = VectorClock::new();
        assert_eq!(clock.get("nonexistent"), 0);
    }

    #[test]
    fn merge_empty_clocks() {
        let mut a = VectorClock::new();
        let b = VectorClock::new();
        a.merge(&b);
        assert!(a.clocks.is_empty());
    }

    #[test]
    fn merge_with_self() {
        let mut clock = VectorClock::new();
        clock.increment("node-a");
        clock.merge(&clock.clone());
        assert_eq!(clock.get("node-a"), 1);
    }

    #[test]
    fn merge_takes_highest() {
        let mut a = VectorClock::new();
        a.increment("node-a");
        a.increment("node-a");
        a.increment("node-a"); // a: {node-a: 3}

        let mut b = VectorClock::new();
        b.increment("node-a");
        b.increment("node-a"); // b: {node-a: 2}

        a.merge(&b);
        assert_eq!(a.get("node-a"), 3); // max(3, 2) = 3
    }

    #[test]
    fn merge_adds_new_nodes() {
        let mut a = VectorClock::new();
        a.increment("node-a"); // a: {node-a: 1}

        let mut b = VectorClock::new();
        b.increment("node-b"); // b: {node-b: 1}

        a.merge(&b);
        assert_eq!(a.get("node-a"), 1);
        assert_eq!(a.get("node-b"), 1);
    }

    #[test]
    fn merge_commutative() {
        let mut a = VectorClock::new();
        a.increment("node-a");
        a.increment("node-a");
        a.increment("node-b");

        let mut b = VectorClock::new();
        b.increment("node-a");
        b.increment("node-c");

        let mut a_then_b = a.clone();
        a_then_b.merge(&b);

        let mut b_then_a = b.clone();
        b_then_a.merge(&a);

        assert_eq!(a_then_b, b_then_a);
    }

    #[test]
    fn serialize_deserialize() {
        let mut clock = VectorClock::new();
        clock.increment("node-a");
        clock.increment("node-b");

        let json = serde_json::to_string(&clock).unwrap();
        let restored: VectorClock = serde_json::from_str(&json).unwrap();
        assert_eq!(clock, restored);
    }

    #[test]
    fn default_trait() {
        let clock = VectorClock::default();
        assert!(clock.clocks.is_empty());
    }
}
