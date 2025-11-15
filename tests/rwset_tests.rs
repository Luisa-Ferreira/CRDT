#[cfg(test)]
mod tests {
   
    use crdt_sss_rs::vclock::VClock;
    use crdt_sss_rs::replica::Replica;
    use crdt_sss_rs::crdt_set::{OpKind, RWSet, Op};
    use std::time::{SystemTime, UNIX_EPOCH};


    //crdt_set tests

    // Test for adding an item
    #[test]
    fn test_add_item() {
        let mut rw_set = RWSet::default();

        // Add an item
        rw_set.add("replica_A", "item1");

        // Verify the item exists and has been added
        let item_state = rw_set.items.get("item1").unwrap();
        assert_eq!(item_state.last.kind, OpKind::Add);
    }

    #[test]
    fn test_apply_add() {
        let mut rw_set = RWSet::default();
        let op_add = Op {
            kind: OpKind::Add,
            wall_time: 100,
            vclock: VClock::default(),
        };
        rw_set.apply("item1", op_add.clone());

        let item_state = rw_set.items.get("item1").unwrap();
        assert_eq!(item_state.last.kind, OpKind::Add);
    }

    // Test for removing an item
    #[test]
    fn test_remove_item() {
        let mut rw_set = RWSet::default();

        // Add and then remove an item
        rw_set.add("replica_A", "item1");
        rw_set.remove("replica_A", "item1");

        // Verify the item is removed
        let item_state = rw_set.items.get("item1").unwrap();
        assert_eq!(item_state.last.kind, OpKind::Remove);
    }

    #[test]
    fn test_apply_remove() {
        let mut rw_set = RWSet::default();
        let op_remove = Op {
            kind: OpKind::Remove,
            wall_time: 100,
            vclock: VClock::default(),
        };
        rw_set.apply("item1", op_remove.clone());

        let item_state = rw_set.items.get("item1").unwrap();
        assert_eq!(item_state.last.kind, OpKind::Remove);
    }

    // Test for merging two sets
    #[test]
    fn test_merge_sets() {
        let mut set1 = RWSet::default();
        let mut set2 = RWSet::default();

        // Add different items to each set
        set1.add("replica_A", "item1");
        set2.add("replica_B", "item2");

        // Merge set2 into set1
        set1.merge(&set2);

        // Verify that both items are now in set1
        assert!(set1.items.contains_key("item1"));
        assert!(set1.items.contains_key("item2"));
    }

    // Test for garbage collection (GC) by TTL
    #[test]
    fn test_gc_ttl() {
        let mut rw_set = RWSet::default();
        let ttl = 5;  // Set TTL to 5 seconds

        rw_set.add("replica_A", "item1");
        rw_set.remove("replica_A", "item1");

        // Simulate the item being expired by 10 seconds (TTL exceeded)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        rw_set.items.get_mut("item1").unwrap().last.wall_time = now - 10;

        rw_set.gc_ttl(ttl);

        // The item should be removed after TTL expiration
        assert!(!rw_set.items.contains_key("item1"));
    }

    // Test anti-entropy verification
    #[test]
    fn test_anti_entropy_verify() {
        let mut rw_set_1 = RWSet::default();
        let mut rw_set_2 = RWSet::default();

        // Add different items to each set
        rw_set_1.add("replica_A", "item1");
        rw_set_2.add("replica_B", "item2");

        // Simulate a remote live set (for anti-entropy verification)
        let mut remote_live = std::collections::HashMap::new();
        remote_live.insert("item1".to_string(), VClock::default()); // Simulating a remote replica that knows about "item1"

        let missing_ops = rw_set_2.anti_entropy_verify(&remote_live, &rw_set_1.frontier);

        // Check that missing_ops contains "item2", since the remote replica doesn't know about it
        assert!(missing_ops.iter().any(|op| op.vclock == rw_set_2.items["item2"].last.vclock));
    }

    #[test]
    fn test_sign_and_verify_op() {
        let sk = generate_ed25519_keypair(); // Função para gerar chave secreta
        let pk = sk.public_key();

        let op = Op {
            kind: OpKind::Add,
            wall_time: 100,
            vclock: VClock::default(),
        };

        let signed_op = sign_op(&sk, op.clone()); // Função que assina a operação
        assert!(verify_op(&pk, &signed_op)); // Verifica se a operação é válida
    }

    #[test]
    fn test_grpc_delta_sync() {
        // Configurar réplicas com mTLS
        let replica_1 = Replica::open_with_mtls("replica_A", PathBuf::from("/tmp"));
        let replica_2 = Replica::open_with_mtls("replica_B", PathBuf::from("/tmp"));

        // Adicionar itens a uma réplica
        replica_1.add("item1").unwrap();

        // Enviar e receber estado via gRPC com mTLS
        replica_1.send_state_via_grpc("replica_B").unwrap();
        replica_2.receive_state_via_grpc().unwrap();

        // Verificar que o estado foi sincronizado
        assert!(replica_2.rset.items.contains_key("item1"));
    }

    #[test]
    fn test_anti_entropy_inconsistent_states() {
        let mut rw_set_1 = RWSet::default();
        let mut rw_set_2 = RWSet::default();

        rw_set_1.add("replica_A", "item1");
        rw_set_2.add("replica_B", "item2");

        let missing_ops = rw_set_1.anti_entropy_verify(&rw_set_2.live_ids(), &rw_set_2.frontier);

        // Verificar que a operação "item2" é listada como falta
        assert!(missing_ops.iter().any(|op| op.vclock == rw_set_2.items["item2"].last.vclock));
    }



}