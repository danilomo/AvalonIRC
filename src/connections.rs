#![allow(unused)]

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio;
use tokio::sync::mpsc::Sender;

use crate::{channels::Channels, errorcodes, messages::UserMessage, user::User};
type ConnectionsMap = HashMap<SocketAddr, Sender<String>>;
type NicksMap = HashMap<String, Sender<String>>;

use anyhow::{anyhow, Context, Result};

const HOST: &str = "olympus";

#[derive(Clone)]
pub struct Connections {
    pub connection_map: Arc<Mutex<ConnectionsMap>>,
    pub nicks_map: Arc<Mutex<NicksMap>>,
    pub channels: Arc<tokio::sync::Mutex<Channels>>,
}

impl Connections {
    pub fn new() -> Connections {
        Connections {
            connection_map: Arc::new(Mutex::new(HashMap::new())),
            nicks_map: Arc::new(Mutex::new(HashMap::new())),
            channels: Arc::new(tokio::sync::Mutex::new(Channels::new())),
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
        message_fn: impl Fn(&str) -> String,
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
            let message_to_send = message_fn(nick);
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

        let message_fn = |nick: &'_ str| format!("{} PRIVMSG {} {}\r\n", &sender, nick, message);

        self.connections
            .send_msg_to_nicks(message_fn, receivers)
            .await;

        Ok(())
    }

    async fn send_msg_to_channel(&mut self, channel: &str, message: &str) -> Result<()> {
        let oclone = self.connections.channels.clone();
        let mut channels = oclone.lock().await;
        let nick = self.user.nick.as_ref().context("NICK is not set")?.clone();
        let nicks = channels.channel_list(channel).filter(|s| **s != *nick);

        let sender = format!(
            ":{}!{}@{}",
            self.user.nick.as_ref().unwrap(),
            self.user.user.as_ref().unwrap(),
            HOST
        );

        let message_fn = |nick: &'_ str| format!("{} PRIVMSG {} {}\r\n", &sender, channel, message);

        self.connections
            .send_msg_to_nicks(message_fn, nicks.map(|s| s.as_str()))
            .await;
        Ok(())
    }

    async fn join_channels(&mut self, channels_names: &[&str], keys: &[&str]) -> Result<()> {
        let oclone = self.connections.channels.clone();
        let mut channels = oclone.lock().await;
        let nick = self.user.nick.as_ref().context("NICK is not set")?.clone();

        for channel_name in channels_names {
            channels.join_user(channel_name, &nick);
            let nicks = channels.channel_list(channel_name);

            let mut nicks_list = String::new();
            for n in channels.channel_list(channel_name) {
                nicks_list.push_str(n);
                nicks_list.push_str(" ");
            }
            nicks_list.pop();

            let sender = format!(
                ":{}!{}@{}",
                self.user.nick.as_ref().unwrap(),
                self.user.user.as_ref().unwrap(),
                HOST
            );
            let message_fn = |nick: &'_ str| format!("{} JOIN {}\r\n", sender, channel_name);
            self.connections
                .send_msg_to_nicks(message_fn,  nicks.map(|s| s.as_str()))
                .await;

            //:odin 331 danilo #rona :No topic is set
            //:odin 353 = #rona :joe danilo
            //:odin 366 = #rona :End of NAMES list

            let response = format!(
                ":{} 331 {} {} :No topic is set \r\n",
                HOST, nick, channel_name,

            );
            self.sender.send(response).await?;
            let response = format!(
                ":{} {} = {} {} \r\n",
                HOST, errorcodes::RPL_NAMREPLY, channel_name, nicks_list,
            );
            self.sender.send(response).await?;
            let response = format!(
                ":{} {} = {} :End of NAMES list \r\n",
                HOST, errorcodes::RPL_ENDOFNAMES, channel_name
            );
            self.sender.send(response).await?;
        }

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
