#[cfg(test)]
#[path = "./messages_test.rs"]
mod messages_test;

#[derive(Debug, PartialEq)]
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

fn split(msg: &str) -> Vec<&str> {
    let parts = msg.split(" ");
    parts.collect::<Vec<&str>>()
}

pub fn parse_message(msg: &str) -> UserMessage {
    let space_index = msg.find(' ').unwrap_or_default();
    let head = &msg[0..space_index];
    let body = &msg[space_index + 1..];

    match head {
        "NICK" => parse_nick(split(body)),
        "USER" => parse_user(split(body)),
        "PASS" => {
            let parts = split(body);
            if let Some(password) = parts.get(0) {
                return UserMessage::Password { password };
            } else {
                UserMessage::InvalidMessage
            }
        }
        "PRIVMSG" => parse_priv_msg(body),
        "QUIT" => {
            let parts = split(body);
            let quit_msg = parts.get(0).map(|str| *str);
            UserMessage::Quit { quit_msg }
        }
        "JOIN" => parse_join_msg(split(body)),
        "PING" => {
            let parts = split(body);
            let server = parts.get(0).map(|str| *str).unwrap_or("");
            UserMessage::Ping {
                server: server.trim(),
            }
        }
        "MODE" => parse_mode_msg(split(body)),

        _ => UserMessage::InvalidMessage,
    }
}

fn parse_nick<'a>(input: Vec<&'a str>) -> UserMessage<'a> {
    match &input[..] {
        [nickname] => UserMessage::Nick {
            nickname: &nickname.trim(),
            hop_count: 0,
        },
        [nickname, hop_str, ..] => {
            let hop = hop_str.trim().parse::<usize>();
            if let Ok(hop_count) = hop {
                UserMessage::Nick {
                    nickname: &nickname.trim(),
                    hop_count,
                }
            } else {
                UserMessage::InvalidMessage
            }
        }
        _ => {
            UserMessage::InvalidMessage
        }
    }
}

fn parse_user<'a>(input: Vec<&'a str>) -> UserMessage<'a> {
    if let [user_name, server_name, host_name, real_name, ..] = &input[..] {
        return UserMessage::User {
            user_name: &user_name.trim(),
            host_name: &host_name.trim(),
            server_name: &server_name.trim(),
            real_name: &real_name.trim(),
        };
    }

    UserMessage::InvalidMessage
}

fn parse_priv_msg<'a>(input: &'a str) -> UserMessage<'a> {
    let space_index = input.find(' ').unwrap_or_default();
    let head = &input[0..space_index];
    let body = &input[space_index + 1..];

    if head == "" {
        return UserMessage::InvalidMessage;
    }

    if head.starts_with("#") {
        return UserMessage::MessageToChannel {
            channel: head,
            message: body,
        };
    }

    let parts = head.split(",");
    let receivers = parts.collect::<Vec<&str>>();
    UserMessage::PrivateMessage {
        receivers: receivers,
        message: body,
    }
}

fn parse_join_msg<'a>(input: Vec<&'a str>) -> UserMessage<'a> {
    match &input[..] {
        [channels] => {
            let parts = channels.split(",");
            let channels = parts.collect::<Vec<&str>>();

            UserMessage::Join {
                channels,
                keys: vec![],
            }
        }
        [channels, keys, ..] => {
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
        [channel] => UserMessage::Mode {
            channel,
            mode: None,
        },
        [channel, mode, ..] => UserMessage::Mode {
            channel,
            mode: Some(*mode),
        },
        _ => UserMessage::InvalidMessage,
    }
}
