use clap::Parser;
use redx_protocol::{Command, parse, tokenize};
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
#[command(version, about = "Interactive redx server shell")]
struct Args {}

fn main() -> io::Result<()> {
    Args::parse();

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line?;
        let tokens = tokenize(&line);

        if !tokens.is_empty() {
            println!(
                "{}",
                tokens
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        let command = match parse(&tokens) {
            Ok(command) => command,
            Err(error) => {
                eprintln!("ERROR: {error}");
                continue;
            }
        };

        println!("PARSED: {command:?}");

        if command == Command::Exit {
            break;
        }
    }

    Ok(())
}
