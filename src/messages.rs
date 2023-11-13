#[derive(Debug)]
pub enum UserMessage<'a> {
    Nick {
        nickname: &'a str,
        hop_count: usize,
    },
    User {
        user_name: &'a str,
        host_name: &'a str,
        server_name: &'a str,
        real_name: &'a str,
    },
    Password {
        password: &'a str,
    },
    PrivateMessage {
        receivers: Vec<&'a str>,
        message: &'a str,
    },
    MessageToChannel {
        channel: &'a str,
        message: &'a str,
    },
    Join {
        channels: Vec<&'a str>,
        keys: Vec<&'a str>,
    },
    Quit {
        quit_msg: Option<&'a str>,
    },
    Ping {
        server: &'a str,
    },
    Mode {
        channel: &'a str,
        mode: Option<&'a str>,
    },
    InvalidMessage,
}

pub fn parse_message(msg: &str) -> UserMessage {
    dbg!(&msg);
    let parts = msg.split(" ");
    let collection = parts.collect::<Vec<&str>>();
    let head = collection.first();

    if let None = head {
        return UserMessage::InvalidMessage;
    }

    let head = *head.unwrap();

    match head {
        "NICK" => parse_nick(collection),
        "USER" => parse_user(collection),
        "PASS" => {
            if let Some(password) = collection.get(1) {
                return UserMessage::Password { password };
            } else {
                UserMessage::InvalidMessage
            }
        }
        "PRIVMSG" => parse_priv_msg(collection),
        "QUIT" => {
            let quit_msg = collection.get(1).map(|str| *str);
            UserMessage::Quit { quit_msg }
        }
        "JOIN" => parse_join_msg(collection),
        "PING" => {
            let server = collection.get(1).map(|str| *str).unwrap_or("");
            UserMessage::Ping { server: server.trim() }
        }
        "MODE" => parse_mode_msg(collection),

        _ => UserMessage::InvalidMessage,
    }
}

fn parse_nick<'a>(input: Vec<&'a str>) -> UserMessage<'a> {
    match &input[..] {
        [_, nickname] => UserMessage::Nick {
            nickname: &nickname.trim(),
            hop_count: 0,
        },
        [_, nickname, hop_str, ..] => {
            let hop = hop_str.parse::<usize>();
            if let Ok(hop_count) = hop {
                UserMessage::Nick {
                    nickname: &nickname.trim(),
                    hop_count,
                }
            } else {
                UserMessage::InvalidMessage
            }
        }
        _ => UserMessage::InvalidMessage,
    }
}

fn parse_user<'a>(input: Vec<&'a str>) -> UserMessage<'a> {
    if let [_, user_name, server_name, host_name, real_name, ..] = &input[..] {
        return UserMessage::User {
            user_name: &user_name.trim(),
            host_name: &host_name.trim(),
            server_name: &server_name.trim(),
            real_name: &real_name.trim(),
        };
    }

    UserMessage::InvalidMessage
}

fn parse_priv_msg<'a>(input: Vec<&'a str>) -> UserMessage<'a> {
    match &input[..] {
        [_, channel, message, ..] if channel.starts_with("#") => {
            UserMessage::MessageToChannel { channel, message }
        }
        [_, recs, message, ..] => {
            let parts = recs.split(",");
            let receivers = parts.collect::<Vec<&str>>();
            UserMessage::PrivateMessage { receivers, message }
        }
        _ => UserMessage::InvalidMessage,
    }
}

fn parse_join_msg<'a>(input: Vec<&'a str>) -> UserMessage<'a> {
    match &input[..] {
        [_, channels] => {
            let parts = channels.split(",");
            let channels = parts.collect::<Vec<&str>>();

            UserMessage::Join {
                channels,
                keys: vec![],
            }
        }
        [_, channels, keys, ..] => {
            let parts = channels.split(",");
            let channels = parts.collect::<Vec<&str>>();

            let parts = keys.split(",");
            let keys = parts.collect::<Vec<&str>>();

            UserMessage::Join { channels, keys }
        }
        _ => UserMessage::InvalidMessage,
    }
}

fn parse_mode_msg<'a>(input: Vec<&'a str>) -> UserMessage<'a> {
    match &input[..] {
        [_, channel] => UserMessage::Mode {
            channel,
            mode: None,
        },
        [_, channel, mode, ..] => UserMessage::Mode {
            channel,
            mode: Some(*mode),
        },
        _ => UserMessage::InvalidMessage,
    }
}
