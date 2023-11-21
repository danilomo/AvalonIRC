use anyhow::Result;
use std::future::Future;
use std::net::SocketAddr;

use crate::connections::Connections;
use crate::messages::parse_message;
use tokio::io::AsyncBufReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::channel;

pub struct Server {
    connections: Connections,
}

impl Server {
    pub fn new() -> Self {
        Server {
            connections: Connections::new(),
        }
    }

    pub async fn start_server(&mut self) -> Result<()> {
        let listener = TcpListener::bind("localhost:6667").await?;

        loop {
            let (socket, addr) = listener.accept().await?;
            let reader = BufReader::new(socket);

            let _ = self.new_connection(addr, reader).await?;
        }
    }

    async fn new_connection(
        &mut self,
        addr: SocketAddr,
        mut reader: BufReader<TcpStream>,
    ) -> Result<()> {
        let (sender, mut receiver) = channel::<String>(10);
        let mut user_connection = self.connections.register_connection(addr, sender)?;

        let future = async move {
            loop {
                let mut message = String::new();

                select! {
                    from_client = reader.read_line(&mut message) => {
                        if let Ok(_) = from_client {
                            let msg = parse_message(&message);
                            dbg!(&msg);
                            let _ = user_connection.handle_message(&msg).await;
                        }
                    },
                    from_server = receiver.recv() => {
                        if let Some(to_send) = from_server {
                            let _ = reader.write(to_send.as_bytes()).await;
                        }
                    }
                };
            }
        };

        tokio::spawn(future);

        Ok(())
    }
}
