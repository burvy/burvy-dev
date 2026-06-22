use anyhow::Result;
use std::time::Duration;
use wtransport::endpoint::IncomingSession;
use tokio::sync::broadcast;
use wtransport::{Endpoint, Identity, ServerConfig, Connection};

/// port forwarded on server pc and network
/// change this port if the server setup changes
const PORT: u16 = 4433;
// certificate and key paths
// theyre not acc here so
// u cant steal my info
// heheheheh
// also they are raw strings so u dont have to do the weird "burvy\\certs\\webtrans" etc
const CERT_PATH: &str = r"C:\burvy\certs\webtrans.burvy.dev\webtrans.burvy.dev-chain.pem";
const KEY_PATH: &str = r"C:\burvy\certs\webtrans.burvy.dev\webtrans.burvy.dev-key.pem";

#[tokio::main]
async fn main() -> Result<()> {
    let identity = Identity::load_pemfiles(CERT_PATH, KEY_PATH).await?;
    let config = ServerConfig::builder()
        .with_bind_default(PORT)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(3))) // dont die on me again
        .build();
    let server = Endpoint::server(config)?; // create the server please dont go wrong
    let (message_text, _) = broadcast::channel::<String>(100);
    loop {
        let incoming = server.accept().await;

        let message_text_clone = message_text.clone();
        // new async task for every browser that we accept (we accept everyone!)
        tokio::spawn(async move {
            if let Err(e) = handle_connection(incoming, message_text_clone).await {
                eprintln!("Error: {}", e);
            }
        });
    }
}

/// handle one connection
async fn handle_connection(incoming: IncomingSession, message_text: broadcast::Sender<String>,) -> Result<()> {
    let connect_req = incoming.await?;
    // this will be useful for different paths like /chat or / something
    println!("requested authority: {}, path: {}", connect_req.authority(), connect_req.path());
    let connection = connect_req.accept().await?; // yayy
    println!("connect request accepted");
    loop {
        // send, receive
        // lets open a bidirectional stream
        // if u dont know what a bidirectional stream is its like
        // walkie talkies except they can talk at the same time
        // ez
        let (mut send, mut recv) = connection.accept_bi().await?;
        // lets reserve 4KB :p
        let mut buffer = vec![0u8; 4096];

        // send bytes or i skip ur stream
        let Some(recv_bytes) = recv.read(&mut buffer).await? else {
            continue;
        };

        // decode the raw bytes into string
        // TODO: change this in the future when we arent just sending strings
        let message = std::str::from_utf8(&buffer[..recv_bytes])?;
        println!("received: {}", message); // hiiii!!!!

        send.write_all(b"hi i saw ur msg hi").await?;
        send.finish().await?; // finish sending the message
    }
}
