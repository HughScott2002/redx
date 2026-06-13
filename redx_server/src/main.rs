use clap::Parser;
use redx_server::{Args, run};
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    run(Args::parse()).await
}
