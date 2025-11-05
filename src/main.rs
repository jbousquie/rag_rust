use std::env;

mod common;
mod indexing;
mod rag_proxy;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [index|proxy]", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "index" => {
            indexing::main().await;
        }
        "proxy" => {
            rag_proxy::main().await;
        }
        _ => {
            eprintln!("Usage: {} [index|proxy]", args[0]);
            std::process::exit(1);
        }
    }
}
