//Save the status of each item (e.g., file) 
// with the last event (add or remove) and its corresponding vector clock.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpKind {
    Add,
    Remove,
}

#[derive(Clone, Debug)]
pub struct Op {
    pub kind: OpKind,
    pub wall_time: u64,  // Use Unix timestamp in seconds 
    pub vclock: VClock,
}

#[derive(Clone, Debug)]
pub struct ItemState {
    pub last: Op,  // The last known operation
}

#[derive(Clone, Debug, Default)]
pub struct RWSet {
    pub items: HashMap<String, ItemState>,  // Map of item ID to state
    pub frontier: VClock,  // The largest clock we've seen so far
}


impl RWSet {
    pub fn apply(&mut self, id: &str, op: Op) {
        if !self.items.contains_key(id) {
            self.items.insert(id.to_string(), ItemState { last: op });
            return;
        }

        let current_state = self.items.get_mut(id).unwrap();
        let current_op = &current_state.last;

        // Compare current op with the new op
        if current_op.vclock.less_equal(&op.vclock) {
            // New operation is greater, apply it
            current_state.last = op.clone();
            self.frontier.merge_max(&op.vclock);
        } else if current_op.vclock.concurrent(&op.vclock) {
            // If the operations are concurrent, apply the remove operation first
            if op.kind == OpKind::Remove {
                current_state.last = op.clone();
                self.frontier.merge_max(&op.vclock);
            } else if current_op.kind == OpKind::Remove {
                // If the current state was removed and the new op is add, apply the add
                current_state.last = op.clone();
                self.frontier.merge_max(&op.vclock);
            }
        }
    }
    pub fn add(&mut self, me: &str, id: &str) {
        let mut new_clock = self.frontier.clone();
        new_clock.inc(me);
        let new_op = Op {
            kind: OpKind::Add,
            wall_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            vclock: new_clock,
        };
        self.apply(id, new_op);
    }

    pub fn remove(&mut self, me: &str, id: &str) {
        let mut new_clock = self.frontier.clone();
        new_clock.inc(me);
        let new_op = Op {
            kind: OpKind::Remove,
            wall_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            vclock: new_clock,
        };
        self.apply(id, new_op);
    }

    pub fn merge(&mut self, other: &RWSet) {
        for (id, other_state) in &other.items {
            let other_op = &other_state.last;
            self.apply(id, other_op.clone());
        }
    }

    pub fn gc_ttl(&mut self, ttl_secs: u64) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let keys_to_remove: Vec<String> = self.items.iter()
            .filter_map(|(id, state)| {
                let op = &state.last;
                if op.kind == OpKind::Remove && now - op.wall_time > ttl_secs {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        for key in keys_to_remove {
            self.items.remove(&key);
        }
    }

    pub fn anti_entropy_verify(&mut self, remote_live: &HashMap<String, VClock>, remote_frontier: &VClock, me: &str) -> Vec<Op> {
        let mut missing_ops = Vec::new();

        for (id, state) in &self.items {
            let op = &state.last;
            if !remote_live.contains_key(id) || !op.vclock.less_equal(remote_frontier) {
                missing_ops.push(op.clone());
            }
        }
        missing_ops
    }
}
