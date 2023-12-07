use std::{collections::HashSet, time::Duration};

use super::*;
use anyhow::Result;
use tokio;

#[tokio::test]
async fn test_connect_to_server() -> Result<()> {
    let addr = start_server().await.addr;
    let bob = TcpStream::connect(addr).await.unwrap();
    let mut bob_stream = BufReader::new(bob);

    bob_stream.write_all(b"NICK bob\r\n").await?;
    bob_stream.write_all(b"USER bob bob bob bob\r\n").await?;
    let mut resp_str = String::new();
    bob_stream.read_line(&mut resp_str).await?;

    assert!(resp_str.contains("Welcome to"));

    assert!(resp_str.contains("bob"));

    Ok(())
}

#[tokio::test]
async fn test_priv_msg() -> Result<()> {
    let addr = start_server().await.addr;
    let bob = TcpStream::connect(addr).await.unwrap();
    let mut bob_stream = BufReader::new(bob);

    let alice = TcpStream::connect(addr).await.unwrap();
    let mut alice_stream = BufReader::new(alice);

    bob_stream.write_all(b"NICK bob\r\n").await?;
    bob_stream.write_all(b"USER bob bob bob bob\r\n").await?;

    alice_stream.write_all(b"NICK alice\r\n").await?;
    alice_stream
        .write_all(b"USER alice alice alice alice\r\n")
        .await?;
    alice_stream
        .write_all(b"PRIVMSG bob eae meu chapa\r\n")
        .await?;

    let mut resp_str = String::new();
    bob_stream.read_line(&mut resp_str).await?; // skips welcome message
    resp_str = String::new();
    bob_stream.read_line(&mut resp_str).await?;

    assert!(resp_str.contains("PRIVMSG bob eae meu chapa"));

    Ok(())
}

#[tokio::test]
async fn test_join_channel() -> Result<()> {
    let info = start_server().await;
    let addr = info.addr;

    let bob = TcpStream::connect(addr).await.unwrap();
    let mut bob_stream = BufReader::new(bob);

    let joe = TcpStream::connect(addr).await.unwrap();
    let mut joe_stream = BufReader::new(joe);

    bob_stream.write_all(b"NICK bob\r\n").await?;
    bob_stream.write_all(b"USER bob bob bob bob\r\n").await?;
    read_line(&mut bob_stream).await?;

    joe_stream.write_all(b"NICK joe\r\n").await?;
    joe_stream.write_all(b"USER joe joe joe joe\r\n").await?;
    read_line(&mut joe_stream).await?;

    bob_stream.write_all(b"JOIN #room1 key\r\n").await?;
    read_line(&mut bob_stream).await?;

    joe_stream.write_all(b"JOIN #room1 key\r\n").await?;
    read_line(&mut bob_stream).await?;
    read_line(&mut joe_stream).await?;

    let channels = info.connections.channels.lock().await;

    let expected_users = vec!["bob", "joe"]
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<_>>();

    let actual_users = channels
        .channel_list("#room1")
        .map(|s| s.to_owned())
        .collect::<HashSet<_>>();

    assert_eq!(expected_users, actual_users);

    Ok(())
}

struct ServerInfo {
    addr: SocketAddr,
    connections: Connections,
}

async fn read_line(stream: &mut BufReader<TcpStream>) -> Result<String> {
    let mut resp = String::new();
    stream.read_line(&mut resp).await?;
    return Ok(resp);
}

async fn start_server() -> ServerInfo {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mut server = Server::new(listener);
    let connections = server.connections.clone();

    tokio::spawn(async move {
        let _ = server.start_server().await;
    });

    ServerInfo { addr, connections }
}
