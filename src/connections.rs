#![allow(unused)]

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc::Sender;

use crate::{channels::Channels, errorcodes, messages::UserMessage, user::User};
type ConnectionsMap = HashMap<SocketAddr, Sender<String>>;
type NicksMap = HashMap<String, Sender<String>>;

use anyhow::{anyhow, Context, Result};

const HOST: &str = "localhost";

#[derive(Clone)]
pub struct Connections {
    connection_map: Arc<Mutex<ConnectionsMap>>,
    nicks_map: Arc<Mutex<NicksMap>>,
    channels: Arc<Mutex<Channels>>,
}

impl Connections {
    pub fn new() -> Connections {
        Connections {
            connection_map: Arc::new(Mutex::new(HashMap::new())),
            nicks_map: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(Mutex::new(Channels::new())),
        }
    }

    pub fn register_connection(
        &mut self,
        address: SocketAddr,
        sender: Sender<String>,
    ) -> Result<UserConnection> {
        let mut map = self.connection_map.lock().unwrap();

        map.insert(address, sender.clone());

        return Ok(UserConnection {
            connections: self.clone(),
            sender,
            user: User::new(),
            authenticated: false,
        });
    }

    fn set_nick_if_available(&mut self, sender: Sender<String>, nick: &str) -> Result<bool> {
        let mut map = self.nicks_map.lock().unwrap();

        if map.contains_key(nick) {
            return Ok(false);
        }

        map.insert(nick.into(), sender);
        return Ok(true);
    }

    async fn send_msg_to_nicks(
        &mut self,
        user: &str,
        message: &str,
        nicks: impl Iterator<Item = &str>,
    ) {
        let senders = {
            let mut senders = vec![];
            let map = self.nicks_map.lock().unwrap();

            for nick in nicks {
                if let Some(sender) = map.get(nick) {
                    let sender = sender.clone();
                    senders.push((nick, sender));
                }
            }
            senders
        };

        for (nick, sender) in senders {
            let message_to_send = format!("{} PRIVMSG {} {}\r\n", user, nick, message);
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
    pub async fn handle_message<'a>(&mut self, message: &UserMessage<'a>) {
        let _ = self.handle_message_aux(message).await;
    }

    async fn handle_message_aux<'a>(&mut self, message: &UserMessage<'a>) -> Result<()> {
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
                self.send_priv_msg(receivers.iter().map(|s| *s), message)
                    .await?
            }
            UserMessage::MessageToChannel { channel, message } => {
                self.send_msg_to_channel(channel, message).await?
            }
            UserMessage::Join { channels, keys } => self.join_channels(channels, keys).await?,
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

    async fn send_priv_msg(
        &mut self,
        receivers: impl Iterator<Item = &str>,
        message: &str,
    ) -> Result<()> {
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

    async fn join_channels(&self, channels_names: &[&str], keys: &[&str]) -> Result<()> {
        let channels = self.connections.channels.lock().unwrap();
        let nick = self.user.nick.as_ref().context("aaaa")?;

        for channel_name in channels_names {
            let nicks = channels.channel_list(channel_name).filter(|s| **s == *nick);
        }

        /*if let (Ok(chans),Some(nick)) = (channels, nick) {
            for channel_name in channels_names {
                let nicks = chans
                    .channel_list(channel_name)
                    .filter();


            }
        }*/

        Ok(())
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
