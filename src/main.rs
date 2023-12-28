use std::iter::zip;

// use std::error::Error;
use anyhow::{Error, anyhow};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::{net::TcpListener, io::AsyncWriteExt};
use tokio::time::{Instant};

const KB: usize = 1024;
const MB: usize = 1024 * KB;
const LEN: usize = 4 * MB;
const MESSAGE: [u8; LEN] = [0; LEN];

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{e}");
    }
}

async fn run() -> Result<(), Error> {
    let mut command = String::new();
    std::io::stdin().read_line(&mut command)?;

    match command.trim() {
        "server" => run_as_server().await,
        "client" => run_as_client().await,
        _ => Err(anyhow!("Error command."))
    }
}

async fn run_as_server() -> Result<(), Error> {
    println!("[SERVER] Waiting for connections...");
    let server = TcpListener::bind("0.0.0.0:37217").await?;
    let (mut client, addr) = server.accept().await?;
    let size = (LEN / MB) as f64;
    let mut speeds = vec![];
    println!("[SERVER] Connected: {}", addr);

    for i in 0..10 {
        let timer = Instant::now();
        client.write_all(&MESSAGE).await?;
        let duration = timer.elapsed().as_secs_f64();
        let speed = size / duration;
        speeds.push(speed);
        println!("[SERVER] Iteration {}: Send to {} at {:.2} MiB/s.", i, addr, speed);
    }

    let speed = speeds.into_iter().sum::<f64>() / 10.0;
    println!("[SERVER] Send to {} at {:.2} MiB/s average.", addr, speed);
    Ok(())
}

async fn run_as_client() -> Result<(), Error> {
    let mut addr = String::new();
    let mut addrs = vec![];
    let mut handles = vec![];
    println!("[CLIENT] Please type ip addresses.");
    loop {
        std::io::stdin().read_line(&mut addr)?;
        if addr.trim().is_empty() {
            break;
        } else {
            addrs.push(addr.trim().to_string() + ":37217");
            addr.clear();
        }
    }

    for i in &addrs {
        handles.push(tokio::spawn(download(i.clone())));
    }

    for (addr, handle) in zip(&addrs, handles) {
        let speed = handle.await??;
        println!("[CLIENT] Recv from {} at {:.2} MiB/s average.", addr, speed);
    }

    Ok(())
}

async fn download(server_addr: String) -> Result<f64, Error> {
    let mut speeds = vec![];
    let mut socket = TcpStream::connect(&server_addr).await?;
    let mut buffer = Vec::new();
    buffer.resize(LEN, 0);
    let size = (LEN / MB) as f64;

    for i in 0..10 {
        let timer = Instant::now();
        let recv_bytes = socket.read_exact(&mut buffer).await?;
        let duration = timer.elapsed().as_secs_f64();

        if recv_bytes != LEN {
            return Err(anyhow!("[CLIENT] {} bytes needed, but receive {}.", LEN, recv_bytes));
        }

        let speed = size / duration;
        speeds.push(speed);
        println!("[CLIENT] Iteration {}: Recv from {} at {:.2} MiB/s.", i, server_addr, speed);
    }

    Ok(speeds.into_iter().sum::<f64>() / 10.0)
}