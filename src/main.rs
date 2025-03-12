use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use std::io::Result;

fn extract_pair(msg: &str) -> Option<String> {
    if let Ok(v) = serde_json::from_str::<Value>(msg) {
        if let Some(pair) = v.get("s") {
            println!("{}", pair);
            return pair.as_str().map(|s| s.to_string());
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000").await?;
    println!("TCP server listening on 0.0.0.0:9000");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("New client connected from {}", addr);

        // Optionally disable Nagle’s algorithm for lower latency.
        if let Err(e) = stream.set_nodelay(true) {
            eprintln!("Failed to set TCP_NODELAY for {}: {:?}", addr, e);
        }

        tokio::spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("Error with client {}: {:?}", addr, e);
            }
            println!("Client {} disconnected", addr);
        });
    }
}

async fn handle_client(stream: TcpStream) -> Result<()> {
    let (read_half, write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            // Connection closed.
            return Ok(());
        }
        let msg_str = line.trim();
        if let Some(pair) = extract_pair(&msg_str) {
            println!("PAIR {:?}", pair);
        } else {
            // println!("No pair")
        }
        // println!("Received from client: {:?}", line.trim());

        // If client sends "ping", respond with "pong"
        if line.trim() == "ping" {
            writer.write_all(b"pong\n").await?;
            writer.flush().await?;
        }
    }
}
