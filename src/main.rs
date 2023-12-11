mod channels;
mod connections;
mod errorcodes;
mod messages;
mod server;
mod user;

use anyhow::Result;
use server::Server;
use tokio::{self, net::TcpListener};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:6667")).await?;
    let mut server = Server::new(listener);

    server.start_server().await?;
    Ok(())
}
