
#[cfg(test)]
mod tests {

    use std::fs; 
    use std::path::PathBuf;
    use crdt_sss_rs::replica::Replica;
   

    #[test]
    fn test_gc_ttl_multiple_replicas() {
         let state_path = PathBuf::from("/tmp").join("state.json");

        // Cleans the state
        if state_path.exists() {
            fs::remove_file(&state_path).unwrap();
        }

        let mut replica_1 = Replica::open("replica_A", PathBuf::from("/tmp")).unwrap();
        let mut replica_2 = Replica::open("replica_B", PathBuf::from("/tmp")).unwrap();

        replica_1.add("item1").unwrap();
        replica_2.add("item1").unwrap();
         
        replica_1.remove("item1").unwrap(); 
        replica_2.remove("item1").unwrap();
        
        
        // Simular a remoção e expiração no tempo
        let ttl = 5;
        replica_1.gc_ttl(ttl).unwrap();
        replica_2.gc_ttl(ttl).unwrap();

        // Verificar que o item foi removido após o TTL
        assert!(!replica_1.rset.items.contains_key("item1"));
        assert!(!replica_2.rset.items.contains_key("item1"));
    }
}