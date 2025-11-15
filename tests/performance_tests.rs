#[cfg(test)]
mod tests {
   
    use crdt_sss_rs::crdt_set::{RWSet};

    #[test]
    fn test_performance_add_remove() {
        let mut rw_set = RWSet::default();

        // Simula a adição e remoção de muitos itens
        for i in 0..1000 {
            let item_name = format!("item{}", i);
            rw_set.add("replica_A", &item_name);
            rw_set.remove("replica_A", &item_name);
        }

        // Verificar que a operação não falhou durante a carga
        assert!(rw_set.items.is_empty());
    }
}
