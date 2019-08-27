extern crate structopt;

use kvs::KvStore;
use std::process;
use structopt::StructOpt;

#[derive(StructOpt)]
enum KvsApp {
    #[structopt(name = "set")]
    Set { key: String, value: String },
    #[structopt(name = "get")]
    Get { key: String },
    #[structopt(name = "rm")]
    Remove { key: String },
}

fn run_app() -> Result<(), ()> {
    let app = KvsApp::from_args();
    let mut kvs = KvStore::new();

    match app {
        KvsApp::Set { key, value } => kvs.set(key, value),
        KvsApp::Get { key } => {
            kvs.get(key);
        }
        KvsApp::Remove { key } => {
            kvs.remove(key);
        }
    }

    eprintln!("unimplemented");
    Err(())
}

fn main() {
    process::exit(match run_app() {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
