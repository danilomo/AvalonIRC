use super::*;

fn assert_messages(msgs: &[&str], expected: &[UserMessage<'_>]) {
    for i in 0..msgs.len() {
        assert_eq!(expected[i], parse_message(msgs[i]),);
    }
}

#[test]
fn test_parse_nick_msg() {
    let msgs = [
        "NICK rona",
        "NICK <>",
        "NICK 12345aeiou__#$",
        "NICK 12345aeiou__#$ 10",
    ];

    let expected = [
        UserMessage::Nick {
            nickname: "rona",
            hop_count: 0,
        },
        UserMessage::Nick {
            nickname: "<>",
            hop_count: 0,
        },
        UserMessage::Nick {
            nickname: "12345aeiou__#$",
            hop_count: 0,
        },
        UserMessage::Nick {
            nickname: "12345aeiou__#$",
            hop_count: 10,
        },
    ];

    assert_messages(&msgs, &expected);
}

#[test]
fn test_parse_join_msg() {
    let msgs = [
        "JOIN #aaa\r\n",
        "JOIN #aaa\r\n",
        "JOIN #aaa",
        "JOIN aaa",
        "JOIN #aaa key1",
        "JOIN #aaa,#bbb key1,key2",
    ];

    let expected = [
        UserMessage::Join {
            channels: vec!["#aaa"],
            keys: vec![],
        },
        UserMessage::Join {
            channels: vec!["#aaa"],
            keys: vec![],
        },
        UserMessage::Join {
            channels: vec!["#aaa"],
            keys: vec![],
        },
        UserMessage::Join {
            channels: vec!["aaa"],
            keys: vec![],
        },
        UserMessage::Join {
            channels: vec!["#aaa"],
            keys: vec!["key1"],
        },
        UserMessage::Join {
            channels: vec!["#aaa", "#bbb"],
            keys: vec!["key1", "key2"],
        },
    ];

    assert_messages(&msgs, &expected);
}

#[test]
fn test_send_msg() {
    let msgs = [
        "PRIVMSG #rona Um dois três de oliveira quatro. !!!123 4%",
        "PRIVMSG rona Um dois três de oliveira quatro. !!!123 4%",
        "PRIVMSG pata,peta,pita,pota Um dois três de oliveira quatro. !!!123 4%",
    ];

    let expected = [
        UserMessage::MessageToChannel {
            channel: "#rona",
            message: "Um dois três de oliveira quatro. !!!123 4%",
        },
        UserMessage::PrivateMessage {
            receivers: vec!["rona"],
            message: "Um dois três de oliveira quatro. !!!123 4%",
        },
        UserMessage::PrivateMessage {
            receivers: vec!["pata", "peta", "pita", "pota"],
            message: "Um dois três de oliveira quatro. !!!123 4%",
        },
    ];

    assert_messages(&msgs, &expected);
}

#[test]
fn test_parse_ping() {
    let msgs = ["PING aaaa", "PING aaaa bbb", "PING"];

    let expected = [
        UserMessage::Ping { server: "aaaa" },
        UserMessage::Ping { server: "aaaa" },
        UserMessage::InvalidMessage,
    ];

    assert_messages(&msgs, &expected);
}

#[test]
fn test_parse_mode() {
    let msgs = ["MODE #channel1 aaa", "MODE #channel1", "MODE"];

    let expected = [
        UserMessage::Mode {
            channel: "#channel1",
            mode: Some("aaa"),
        },
        UserMessage::Mode {
            channel: "#channel1",
            mode: None,
        },
        UserMessage::InvalidMessage,
    ];

    assert_messages(&msgs, &expected);
}

#[test]
fn test_pass_msg() {
    let msgs = ["PASS 12345", "PASS 2893749sdofuoui)*(&)(&#H", "PASS"];

    let expected = [
        UserMessage::Password { password: "12345" },
        UserMessage::Password {
            password: "2893749sdofuoui)*(&)(&#H",
        },
        UserMessage::InvalidMessage,
    ];

    assert_messages(&msgs, &expected);
}
