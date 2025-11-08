#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::path::PathBuf;
    use crdt_sss_rs::vclock::VClock;
    use crdt_sss_rs::replica::Replica;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
   fn test_cli_add() {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("--replica")
        .arg("replica_A")
        .arg("--root")
        .arg("/tmp")
        .arg("add")
        .arg("item1")
        .output()
        .expect("Failed to execute command");


        // Verifica se o comando foi bem-sucedido
        assert!(output.status.success(), "Command failed with status: {:?}", output.status);
        assert!(String::from_utf8_lossy(&output.stdout).contains("OK add"));
    }

    #[test]
    fn test_cli_rm() {
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")        
            .arg("--replica")
            .arg("replica_A")
            .arg("--root")
            .arg("/tmp")
            .arg("rm")
            .arg("item1")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("OK rm"));
    }

    #[test]
    fn test_send_and_receive_state() {
        let mut replica_1 = Replica::open("replica_A", PathBuf::from("/tmp")).unwrap();
        let mut replica_2 = Replica::open("replica_B", PathBuf::from("/tmp")).unwrap();

        replica_1.add("item1").unwrap();
        let inbox_b = PathBuf::from("/tmp").join("replica_B").join("inbox");
        replica_1.send_state_to(inbox_b).unwrap();

        // Simulate replica_2 receiving the state
        replica_2.receive_and_merge().unwrap();

        // Ensure replica_2 has received the correct state
        assert!(replica_2.rset.items.contains_key("item1"));
    }

    #[test]
    fn test_gc_ttl_in_replica() {
        let mut replica = Replica::open("replica_A", PathBuf::from("/tmp")).unwrap();

        replica.add("item1").unwrap();
        replica.remove("item1").unwrap();

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        replica.rset.items.get_mut("item1").unwrap().last.wall_time = now - 10;

        replica.gc_ttl(5).unwrap();

        assert!(!replica.rset.items.contains_key("item1"));
    }

    #[test]
    fn test_vclock_inc() {
        let mut vclock = VClock::default();
        vclock.inc("replica_A");
        
        assert_eq!(vclock.0.get("replica_A").unwrap(), &1);
    }

    #[test]
    fn test_vclock_merge() {
        let mut vclock_1 = VClock::default();
        let mut vclock_2 = VClock::default();

        vclock_1.inc("replica_A");
        vclock_2.inc("replica_B");

        vclock_1.merge_max(&vclock_2);

        assert_eq!(vclock_1.0.get("replica_A").unwrap(), &1);
        assert_eq!(vclock_1.0.get("replica_B").unwrap(), &1);
    }

    #[test]
    fn test_vclock_less_equal() {
        let mut vclock_1 = VClock::default();
        let mut vclock_2 = VClock::default();

        vclock_1.inc("replica_A");
        vclock_2.inc("replica_A");

        assert!(vclock_1.less_equal(&vclock_2));
    }

    #[test]
    fn test_vclock_concurrent() {
        let mut vclock_1 = VClock::default();
        let mut vclock_2 = VClock::default();

        vclock_1.inc("replica_A");
        vclock_2.inc("replica_B");

        assert!(vclock_1.concurrent(&vclock_2));
    }
}
