use anyhow::Result;
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

#[cfg(test)]
#[path = "./server_test.rs"]
mod server_test;

pub struct Server {
    pub connections: Connections,
    pub listener: TcpListener,
}

impl Server {
    pub fn new(listener: TcpListener) -> Self {
        Server {
            connections: Connections::new(),
            listener,
        }
    }

    pub async fn start_server(&mut self) -> Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            let reader = BufReader::new(socket);

            tokio::select! {
                _ = self.new_connection(addr, reader) => {}
                /*_ = tokio::signal::ctrl_c() => {
                    return Ok(());
                }*/
            };
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
