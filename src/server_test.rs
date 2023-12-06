use super::*;
use anyhow::Result;
use tokio;

#[tokio::test]
async fn test_connect_to_server() -> Result<()> {
    let addr = start_server().await;
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
    let addr = start_server().await;
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
    Ok(())
}

async fn start_server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let mut server = Server::new(listener);
        let _ = server.start_server().await;
    });

    addr
}
