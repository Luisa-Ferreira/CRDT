use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Vector Clock structure
/// Keeps track of the number of operations observed by each replica.
/// Example: {"A": 5, "B": 3} means replica A performed 5 operations and B performed 3.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VClock(pub HashMap<String, u64>);

impl VClock {
    /// Increments the counter of the given replica ID.
    pub fn inc(&mut self, replica_id: &str) {
        *self.0.entry(replica_id.to_string()).or_insert(0) += 1;
    }

    /// Merges this clock with another by taking the maximum counter value per replica.
    /// Ensures monotonicity and eventual convergence.
    pub fn merge_max(&mut self, other: &VClock) {
        for (replica, value) in &other.0 {
            let entry = self.0.entry(replica.clone()).or_insert(0);
            if *entry < *value {
                *entry = *value;
            }
        }
    }

    /// Returns true if this clock is less than or equal to the other clock,
    /// meaning it has seen no events that the other hasn't.
    pub fn less_equal(&self, other: &VClock) -> bool {
        for (replica, my_val) in &self.0 {
            let other_val = other.0.get(replica).cloned().unwrap_or(0);
            if my_val > &other_val {
                return false;
            }
        }
        true
    }

    /// Returns true if this and the other clock are concurrent —
    /// neither dominates the other (each has unseen events from the other).
    pub fn concurrent(&self, other: &VClock) -> bool {
        !self.less_equal(other) && !other.less_equal(self)
    }
}