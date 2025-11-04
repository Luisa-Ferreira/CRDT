use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crdt_sss_rs::replica::Replica;

#[derive(Parser)]
#[command(version, about="CRDT-SSS (begineer) with Vector Clock + TTL + verify in sync")]
struct Cli {
    /// replica ID (ex.: A ou B)
    #[arg(long)]
    replica: String,

    #[arg(long)]
    root: PathBuf,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Add { id: String },
    Rm { id: String },
    List,
    Sync { #[arg(long)] to: Option<PathBuf>, #[arg(long)] recv: bool },
    Gc { #[arg(long, default_value_t = 3600)] ttl_secs: i64 },
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    let mut r = Replica::open(&cli.replica, cli.root.clone())?;

    match cli.cmd {
        Cmd::Add { id } => { r.add(&id)?; println!("OK add {}", id); }
        Cmd::Rm { id } => { r.remove(&id)?; println!("OK rm {}", id); }
        Cmd::List => { println!("{:?}", r.list()); }
        Cmd::Sync { to, recv } => {
            if let Some(dst) = to { r.send_state_to(dst)?; println!("OK enviado"); }
            if recv { r.receive_and_merge()?; println!("OK recebido/merge"); }
        }
        Cmd::Gc { ttl_secs } => { r.gc_ttl(ttl_secs)?; println!("OK gc ttl={}s", ttl_secs); }
    }
    Ok(())
}
