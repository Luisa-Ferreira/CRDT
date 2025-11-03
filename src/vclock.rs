import java.util.HashMap;
//A map (HashMap<String, u64>) that stores the counter for each replica.
//Example: { "A": 5, "B": 3 } means that A performed 5 operations and B performed 3.

class VectorClock {
    private HashMap<String, u64> counterReplica;

    public VectorClock() {
        this.counterReplica = new HashMap<String, u64>();
    }

    public void increment(String replicaId) {
        this.counterReplica.put(replicaId, this.counterReplica.getOrDefault(replicaId, 0) + 1);
    }

    public void merge_max(VectorClock other) {
        //gets the other vector and compares with the actual one, keeping the maximum value (recent one) in each replica
        for (String key : other.getCounterReplica().keySet()) {
            u64 otherValue = other.getCounterReplica().get(key);
            u64 thisValue = this.counterReplica.getOrDefault(key, 0);
            this.counterReplica.put(key, Math.max(thisValue, otherValue));
        }
    }

    public less_equal(VectorClock a, VectorClock b) {
        ''' return true if, for example, A (other) saw everything that B have seen. A is updated about B'''
        for (String key : a.getCounterReplica().keySet()) {
            u64 aValue = a.getCounterReplica().get(key);
            u64 bValue = b.getCounterReplica().getOrDefault(key, 0);
            if (aValue > bValue) {
                return false;
            }
        }
        return true;
    }

    public concurrent (VectorClock a, VectorClock b) {
        // A os updated about B and B is updated about A. Both have seens the exact same operations
        return !less_equal(a, b) && !less_equal(b, a);
    }

    public HashMap<String, u64> getCounterReplica() {
        return this.counterReplica;
    }
}