#![allow(unused)]

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc::Sender;

use crate::{errorcodes, messages::UserMessage, user::User};
type ConnectionsMap = HashMap<SocketAddr, Sender<String>>;
type NicksMap = HashMap<String, Sender<String>>;

use anyhow::{anyhow, Result};

const HOST: &str = "localhost";

#[derive(Clone)]
pub struct Connections {
    connection_map: Arc<Mutex<ConnectionsMap>>,
    nicks_map: Arc<Mutex<NicksMap>>,
}

impl Connections {
    pub fn new() -> Connections {
        Connections {
            connection_map: Arc::new(Mutex::new(HashMap::new())),
            nicks_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_connection(
        &mut self,
        address: SocketAddr,
        sender: Sender<String>,
    ) -> Result<UserConnection> {
        if let Ok(mut map) = self.connection_map.lock() {
            map.insert(address, sender.clone());

            return Ok(UserConnection {
                connections: self.clone(),
                sender,
                user: User::new(),
                authenticated: false,
            });
        }

        Err(anyhow!("Failed to obtain mutex for connection_map"))
    }

    fn set_nick_if_available(&mut self, sender: Sender<String>, nick: &str) -> Result<bool> {
        if let Ok(mut map) = self.nicks_map.lock() {
            if map.contains_key(nick) {
                return Ok(false);
            }

            map.insert(nick.into(), sender);
            return Ok(true);
        }

        Err(anyhow!("Failed to obtain mutex for connectino_map"))
    }

    async fn send_msg_to_nicks(&mut self, user: &str, message: &str, nicks: &[&str]) {
        let mut senders = vec![];
        if let Ok(map) = self.nicks_map.lock() {
            for nick in nicks {
                if let Some(sender) = map.get(*nick) {
                    let sender = sender.clone();
                    senders.push((nick, sender));
                }
            }
        }

        for (nick, sender) in senders {
            let message_to_send = format!(
                "{} PRIVMSG {} {}\r\n",
                user,
                nick,
                message
            );
            sender.send(message_to_send).await;
        }

    }
}

pub struct UserConnection {
    connections: Connections,
    sender: Sender<String>,
    user: User,
    authenticated: bool,
}

impl UserConnection {
    pub async fn handle_message<'a>(&mut self, message: &UserMessage<'a>) -> Result<()> {
        match message {
            UserMessage::Nick {
                nickname,
                hop_count,
            } => {
                self.set_nick(nickname, *hop_count).await?;
                self.check_authenticated().await?
            }
            UserMessage::User {
                user_name,
                host_name,
                server_name,
                real_name,
            } => {
                self.set_user(user_name, host_name, server_name, real_name);
                self.check_authenticated().await?
            }
            UserMessage::Password { password } => self.set_password(password).await?,
            UserMessage::PrivateMessage { receivers, message } => {
                self.send_priv_msg(receivers, message).await?
            }
            UserMessage::MessageToChannel { channel, message } => {
                self.send_msg_to_channel(channel, message).await?
            }
            UserMessage::Join { channels, keys } => self.join_channel(channels, keys).await?,
            UserMessage::Quit { quit_msg } => self.quit(*quit_msg).await?,
            UserMessage::Ping { server } => self.ping(server).await?,
            UserMessage::Mode { channel, mode } => self.set_mode(channel, *mode).await?,
            UserMessage::InvalidMessage => {}
        }

        Ok(())
    }

    async fn check_authenticated(&mut self) -> Result<()> {
        if self.authenticated {
            return Ok(());
        }

        if let (Some(nick), Some(_)) = (&self.user.nick, &self.user.user) {
            let welcome_msg = format!(
                ":{} 001 {} :Welcome to the Internet Relay Network, {}!\r\n",
                HOST, nick, nick
            );
            dbg!(&welcome_msg);
            self.sender.send(welcome_msg).await?;
            self.authenticated = true;
        }

        Ok(())
    }

    async fn set_nick(&mut self, nickname: &str, _hop_count: usize) -> Result<()> {
        let sender = self.sender.clone();
        let result = self.connections.set_nick_if_available(sender, nickname)?;

        if !result {
            let message = format!(
                ":{} {} * {} :Nickname already in use\r\n",
                HOST,
                errorcodes::ERR_ALREADYREGISTRED,
                nickname
            );
            dbg!(&message);
            self.sender.send(message).await?;
        } else {
            self.user.nick = Some(nickname.into());
        }

        Ok(())
    }

    fn set_user(&mut self, user_name: &str, host_name: &str, _server_name: &str, real_name: &str) {
        self.user.user = Some(user_name.into());
        self.user.host = Some(host_name.into());
        self.user.full_name = Some(real_name.into());
    }

    async fn set_password(&self, password: &str) -> Result<()> {
        todo!()
    }

    async fn send_priv_msg(&mut self, receivers: &[&str], message: &str) -> Result<()> {
        /*
        let message_to_send = format!(
            "PRIVMSG {} {}\r\n",
            self.user.nick.as_ref().unwrap(),
            message
        );
        */
        let sender = format!(
            ":{}!{}@{}",
            self.user.nick.as_ref().unwrap(),
            self.user.user.as_ref().unwrap(),
            HOST
        );

        self.connections
            .send_msg_to_nicks(&sender, message, receivers)
            .await;

        Ok(())
    }

    async fn send_msg_to_channel(&self, channel: &str, message: &str) -> Result<()> {
        todo!()
    }

    async fn join_channel(&self, channels: &[&str], keys: &[&str]) -> Result<()> {
        todo!()
    }

    async fn quit(&self, quit_msg: Option<&str>) -> Result<()> {
        todo!()
    }

    async fn ping(&self, server: &str) -> Result<()> {
        let pong = format!(":{} {} {}\r\n", HOST, HOST, server);
        self.sender.send(pong).await?;

        Ok(())
    }

    async fn set_mode(&self, channel: &str, mode: Option<&str>) -> Result<()> {
        todo!()
    }
}
