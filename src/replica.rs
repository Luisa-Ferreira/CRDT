use std::path::PathBuf;
use std::fs;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::crdt_set::RWSet;
use crate::vclock::VClock;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Snapshot {
    pub me: String,
    pub rset: RWSet,
}

pub struct Replica {
    pub me: String,
    root: PathBuf,         // ex.: replicas/A
    inbox: PathBuf,        // ex.: replicas/A/inbox
    state_path: PathBuf,   // ex.: replicas/A/state.json
    pub rset: RWSet,
}

impl Replica {
    pub fn open(me: &str, root: PathBuf) -> std::io::Result<Self> {
        let inbox = root.join("inbox");
        fs::create_dir_all(&inbox)?;
        let state_path = root.join("state.json");
        let rset = if state_path.exists() {
            let txt = fs::read_to_string(&state_path)?;
            serde_json::from_str::<Snapshot>(&txt).map(|s| s.rset).unwrap_or_default()
        } else { RWSet::default() };

        Ok(Self { me: me.to_string(), root, inbox, state_path, rset })
    }

    pub fn save(&self) -> std::io::Result<()> {
        let snap = Snapshot { me: self.me.clone(), rset: self.rset.clone() };
        fs::write(&self.state_path, serde_json::to_string_pretty(&snap).unwrap())
    }

    pub fn add(&mut self, id: &str) -> std::io::Result<()> {
        self.rset.add(&self.me, id);
        self.save()
    }

    pub fn remove(&mut self, id: &str) -> std::io::Result<()> {
        self.rset.remove(&self.me, id);
        self.save()
    }

    pub fn list(&self) -> Vec<String> { self.rset.live_ids() }

    pub fn send_state_to(&self, other_inbox: PathBuf) -> std::io::Result<()> {
        fs::create_dir_all(&other_inbox)?;
        #[derive(Serialize)]
        struct Wire<'a> {
            from: &'a str,
            frontier: &'a VClock,
            live: HashMap<String, VClock>,
            full: &'a RWSet, 
        }

        let mut live_map: HashMap<String, VClock> = HashMap::new();
        for id in self.rset.live_ids() {
            if let Some(st) = self.rset.items.get(&id) {
                    live_map.insert(id, st.last.vclock.clone());
            }
        }
        let payload = Wire {
            from: &self.me,
            frontier: &self.rset.frontier,
            live: live_map,
            full: &self.rset,
        };
        let tmp = other_inbox.join("incoming.json");
        fs::write(&tmp, serde_json::to_string(&payload).unwrap())?;
        eprintln!("State sent to: {:?}", tmp);

        let finalp = other_inbox.join(format!("state_from_{}.json", self.me));
        fs::rename(tmp, finalp)?;

        Ok(())
    }

    pub fn receive_and_merge(&mut self) -> std::io::Result<()> {

        if !self.inbox.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Inbox directory not found"));
        }
        let inbox_dir = &self.inbox;
        eprintln!("Checking inbox at: {:?}", inbox_dir); 

        let entries: Vec<_> = fs::read_dir(inbox_dir)?
            .filter_map(Result::ok) // Filtra erros, se houver
            .collect();

        if entries.is_empty() {
            eprintln!("Inbox directory is empty.");
        }

        for entry in entries {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) == Some("json") {
                let txt = fs::read_to_string(&p)?;
                // struture received
                #[derive(Deserialize)]
                struct Wire {
                    frontier: VClock,
                    live: HashMap<String, VClock>,
                    full: RWSet,
                }
                let w: Wire = serde_json::from_str(&txt).unwrap();

                self.rset.anti_entropy_verify(&w.live, &w.frontier);

                let cloned = w.full.clone();
                self.rset.merge(&cloned);

                fs::remove_file(&p)?;
            }
        }
        self.save()
    }

    pub fn gc_ttl(&mut self, ttl_secs: i64) -> std::io::Result<()> {
        //if a kind= delete, wall_time is older than ttl_secs, remove it from the set
        self.rset.gc_ttl(ttl_secs.try_into().unwrap());
        self.save()
    }
    
}
