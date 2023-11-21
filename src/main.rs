mod channels;
mod connections;
mod errorcodes;
mod messages;
mod server;
mod user;

use anyhow::Result;
use server::Server;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let mut server = Server::new();

    server.start_server().await?;
    Ok(())
}
