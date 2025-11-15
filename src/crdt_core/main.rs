use clap::{Parser, Subcommand};
use std::{path::PathBuf, net::SocketAddr};
use crdt_sss_rs::replica::Replica;
use crdt_sss_rs::net::start_http_server;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[command(version, about="CRDT-SSS (begineer) with Vector Clock + TTL + verify in sync")]
struct Cli {
    /// replica ID (ex.: A or B)
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
    Serve { #[arg(long, default_value = "127.0.0.1:8080")] listen: String },
    SyncHttp { #[arg(long)] to: String },
    Gc { #[arg(long, default_value_t = 3600)] ttl_secs: i64 },
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let mut r = Replica::open(&cli.replica, cli.root.clone())?;

    match cli.cmd {
        Cmd::Add { id } => { r.add(&id)?; println!("OK add {}", id); }
        Cmd::Rm { id } => { r.remove(&id)?; println!("OK rm {}", id); }
        Cmd::List => { println!("{:?}", r.list()); }
        Cmd::Serve { listen } => {
            let addr: SocketAddr = listen.parse().expect("invalid addr");
            let shared = Arc::new(Mutex::new(r));
            start_http_server(shared, addr).await;
        }
        Cmd::SyncHttp { to } => {
            let r = Replica::open(&cli.replica, cli.root.clone())?;
            match r.send_state_http(&to).await {
                Ok(_) => println!("OK enviado HTTP"),
                Err(e) => eprintln!("Erro: {e:?}"),
            }
        }
        Cmd::Gc { ttl_secs } => { r.gc_ttl(ttl_secs)?; println!("OK gc ttl={}s", ttl_secs); }
    }
    Ok(())
}
