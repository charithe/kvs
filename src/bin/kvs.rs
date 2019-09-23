extern crate structopt;

use kvs::KvStore;
use std::path::Path;
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

fn run_app() -> kvs::Result<()> {
    let app = KvsApp::from_args();
    let mut kvs = KvStore::open(Path::new("data.log"))?;

    match app {
        KvsApp::Set { key, value } => kvs.set(key, value),
        KvsApp::Get { key } => kvs
            .get(key)
            .map(|v| println!("{}", v.unwrap_or_else(|| "Key not found".to_string()))),
        KvsApp::Remove { key } => kvs.remove(key),
    }
}

fn main() {
    process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            println!("{}", err);
            1
        }
    });
}
