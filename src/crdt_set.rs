//Save the status of each item (e.g., file) 
// with the last event (add or remove) and its corresponding vector clock.

Enum OpKind {
    ADD,
    REMOVE
}

Struct Op {
    String kind; // add or remove
    Time wall_time;
    VectorClock vclock;
}

Struct ItemState {
    Op last;//last event known
}

Struct RWSet {
    HashMap<String, ItemState> items; // Map of item ID to its state
    VectorClock frontier; // bigger clock seen so far
}

private apply (id, op){
    if (!this.items.containsKey(id)) {
        this.items.put(id, new ItemState(op));
        return;
    }

    ItemState currentState = this.items.get(id);
    if (VectorClock.less_equal(currentState.last.vclock, op.vclock)) {
        this.items.put(id, new ItemState(op));
        this.frontier.merge_max(op.vclock);
    }else if (VectorClock.concurrent(currentState.last.vclock, op.vclock)) {
        if (op.kind == OpKind.REMOVE) {
            this.items.put(id, new ItemState(op));
            this.frontier.merge_max(op.vclock);
        }else if (currentState.last.kind == OpKind.REMOVE) {
            this.items.put(id, new ItemState(op));
            this.frontier.merge_max(op.vclock);
        }
    }
}

add(me, id){
    VectorClock newClock = this.frontier.clone();
    newClock.increment(me);
    Op newOp = new Op(OpKind.ADD, Time.now(), newClock);
    this.apply(id, newOp);
}

remove(me, int id){
    VectorClock newClock = this.frontier.clone();
    newClock.increment(me);
    Op newOp = new Op(OpKind.REMOVE, Time.now(), newClock);
    this.apply(id, newOp);
}

merge(other){
    //une the two sets by applying the last operation of each item from the other set
    for (String id : other.items.keySet()) {
        Op otherOp = other.items.get(id).last;
        this.apply(id, otherOp);
    }
}

gc_ttl(ttl_secs){
    Time now = Time.now();
    for (String id : this.items.keySet()) {
        Op op = this.items.get(id).last;
        //remove old tombstones
        if (op.kind == OpKind.REMOVE && now - op.wall_time > ttl_secs) {
            this.items.remove(id);
        }
    }
}

// garantee that "zombie" items are not created during anti-entropy
anti_entropy_verify(remote_live, remote_frontier, me){
    // Verify which operations the remote replica is missing
    List<Op> missing_ops = new List<Op>();
    for (String id : this.items.keySet()) {
        Op op = this.items.get(id).last;
        if (!VectorClock.less_equal(op.vclock, remote_frontier)) {
            missing_ops.add(op);
        }
    }
    return missing_ops;
}