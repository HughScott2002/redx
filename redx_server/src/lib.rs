use clap::Parser;
use redx_protocol::{DecodeResult, RespCommand, decode};
use std::{io, net::SocketAddr};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:6379";
const READ_BUFFER_SIZE: usize = 4096;
const OK_RESPONSE: &[u8] = b"+OK\r\n";

#[derive(Parser, Debug)]
#[command(version, about = "Redis-compatible logging server for redx")]
pub struct Args {
    #[arg(long, default_value = DEFAULT_LISTEN_ADDR)]
    pub listen_addr: SocketAddr,
}

pub async fn run(args: Args) -> io::Result<()> {
    let listener = TcpListener::bind(args.listen_addr).await?;
    println!("listening on {}", listener.local_addr()?);
    accept_loop(listener).await
}

async fn accept_loop(listener: TcpListener) -> io::Result<()> {
    loop {
        let (stream, peer) = listener.accept().await?;
        println!("accepted peer={peer}");

        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, peer).await {
                eprintln!("peer={peer} io_error={error}");
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream, peer: SocketAddr) -> io::Result<()> {
    let mut buffered = Vec::new();
    let mut chunk = [0; READ_BUFFER_SIZE];

    loop {
        let bytes_read = stream.read(&mut chunk).await?;

        if bytes_read == 0 {
            println!("closed peer={peer}");
            return Ok(());
        }

        buffered.extend_from_slice(&chunk[..bytes_read]);

        while !buffered.is_empty() {
            match decode(&buffered) {
                Ok(DecodeResult::Complete { frame, consumed }) => {
                    let command = match RespCommand::from_frame(&frame) {
                        Ok(command) => command,
                        Err(error) => {
                            eprintln!("peer={peer} protocol_error={error}");
                            return Ok(());
                        }
                    };

                    log_received_command(peer, &command);
                    stream.write_all(OK_RESPONSE).await?;
                    buffered.drain(..consumed);
                }
                Ok(DecodeResult::Incomplete) => break,
                Err(error) => {
                    eprintln!("peer={peer} protocol_error={error}");
                    return Ok(());
                }
            }
        }
    }
}

fn log_received_command(peer: SocketAddr, command: &RespCommand) {
    let name = command.name_lossy();
    let arguments = command.arguments_lossy();

    match command.to_command() {
        Ok(parsed) => {
            println!("peer={peer} command={name} args={arguments:?} parsed={parsed:?}");
        }
        Err(error) => {
            println!("peer={peer} command={name} args={arguments:?} parse_error={error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{io::AsyncWriteExt, task::JoinHandle};

    #[test]
    fn parses_default_listen_addr() {
        let args = Args::parse_from(["redx_server"]);

        assert_eq!(args.listen_addr, DEFAULT_LISTEN_ADDR.parse().unwrap());
    }

    #[tokio::test]
    async fn acknowledges_single_ping_command() {
        let (addr, server_task) = spawn_single_connection_server().await;
        let mut client = TcpStream::connect(addr).await.unwrap();

        client.write_all(b"*1\r\n$4\r\nPING\r\n").await.unwrap();

        let mut response = [0; OK_RESPONSE.len()];
        client.read_exact(&mut response).await.unwrap();

        assert_eq!(&response, OK_RESPONSE);

        drop(client);
        server_task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn acknowledges_multiple_pipelined_commands() {
        let (addr, server_task) = spawn_single_connection_server().await;
        let mut client = TcpStream::connect(addr).await.unwrap();

        client
            .write_all(b"*1\r\n$4\r\nPING\r\n*2\r\n$4\r\nKEYS\r\n$1\r\n*\r\n")
            .await
            .unwrap();

        let mut response = [0; OK_RESPONSE.len() * 2];
        client.read_exact(&mut response).await.unwrap();

        assert_eq!(&response, b"+OK\r\n+OK\r\n");

        drop(client);
        server_task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn closes_connection_on_malformed_resp() {
        let (addr, server_task) = spawn_single_connection_server().await;
        let mut client = TcpStream::connect(addr).await.unwrap();

        client.write_all(b"!1\r\n").await.unwrap();

        let mut response = [0; 1];
        let bytes_read = client.read(&mut response).await.unwrap();

        assert_eq!(bytes_read, 0);

        drop(client);
        server_task.await.unwrap().unwrap();
    }

    async fn spawn_single_connection_server() -> (SocketAddr, JoinHandle<io::Result<()>>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let (stream, peer) = listener.accept().await?;
            handle_connection(stream, peer).await
        });

        (addr, task)
    }
}
