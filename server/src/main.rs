use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use wtransport::endpoint::IncomingSession;
use wtransport::{Connection, Endpoint, Identity, ServerConfig};

/// port forwarded on server pc and network
/// change this port if the server setup changes
const PORT: u16 = 4433;

/// biggest message you can send in bytes (self explanatory)
const MAX_MESSAGE_BYTES: usize = 4096;
// certificate and key paths
// theyre not acc here so
// u cant steal my info
// heheheheh
// also they are raw strings so u dont have to do the weird "burvy\\certs\\webtrans" etc
const CERT_PATH: &str = r"C:\burvy\certs\webtrans.burvy.dev\webtrans.burvy.dev-chain.pem";
const KEY_PATH: &str = r"C:\burvy\certs\webtrans.burvy.dev\webtrans.burvy.dev-key.pem";

/// shared server state
/// count clients and keep track of what room they are in
/// then we can send messages and manage them easier
#[derive(Default)]
struct ServerState {
    client_counter: u64,
    rooms: HashMap<String, HashMap<u64, Outbox>>,
}

/// what the client wants to send
type Outbox = mpsc::UnboundedSender<String>;

/// what the client is receiving
type Inbox = mpsc::UnboundedReceiver<String>;
/// everyone must share the server and therefore must read this shared state
/// which is the ServerState
type SharedState = Arc<Mutex<ServerState>>;

#[tokio::main]
async fn main() -> Result<()> {
    let identity = Identity::load_pemfiles(CERT_PATH, KEY_PATH).await?;
    let config = ServerConfig::builder()
        .with_bind_default(PORT)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(3))) // dont die on me again
        .build();
    let server = Endpoint::server(config)?; // create the server please dont go wrong
    let state: SharedState = Arc::new(Mutex::new(ServerState::default())); // the weird shared data type above
    loop {
        let incoming = server.accept().await;
        let state_clone = state.clone();
        // new async task for every browser that we accept (we accept everyone!)
        tokio::spawn(async move {
            if let Err(e) = handle_connection(incoming, state_clone).await {
                eprintln!("Error: {}", e);
            }
        });
    }
}

/// handle one connection
/// this connection is able to access the shared server state
async fn handle_connection(incoming: IncomingSession, state: SharedState) -> Result<()> {

    // process and accept a connection
    let connect_req = incoming.await?;
    println!("requested authority: {}, path: {}", connect_req.authority(), connect_req.path());
    // decide what room they are in
    let room = connect_req.path().to_string();
    let connection = connect_req.accept().await?; // yayy
    println!("connection request accepted");

    // join them to taht room
    let (client_id, mut inbox) = join_room(&state, &room).await;

    // start letting them send stuff
    let send_connection = connection.clone();
    let send_task = tokio::spawn(async move {
        while let Some(message) = inbox.recv().await {
            if let Err(e) = send_message_to_client(&send_connection, &message).await {
                eprintln!("Error: {}", e);
                break;
            }
        }
    });
    // read messages
    let read_result = read_messages_from_client(connection, state.clone(), &room, client_id).await;
    // remove them from the room if they leave
    leave_room(&state, &room, client_id).await;
    send_task.abort();
    read_result
}

/// inserting clients into a room and setup room
async fn join_room(
    state: &SharedState,
    room: &str,
) -> (u64, Inbox) {
    // a private channel for this client
    let (outbox, inbox) = mpsc::unbounded_channel();
    // lock the state right now so we can edit stuff
    let mut state = state.lock().await;

    let client_id = state.client_counter;
    // set the next client id to be unique since we are using this current one now
    state.client_counter += 1;

    // find this room, or create it if it doesnt exist and put the client outbox in it in either case
    state.rooms.entry(room.to_string()).or_default().insert(client_id, outbox);
    (client_id, inbox)
}

/// cleanup clients and rooms when leaving
async fn leave_room(state: &SharedState, room: &str, client_id: u64) {
    // lock the state so we can edit stuff again
    // this may take a bit if theres a bunch of waiting locks btw
    let mut state = state.lock().await;

   // does this room really exist??
    let Some(this_room) = state.rooms.get_mut(room) else {
        return;
    };

    // remove the client from this room when they leave
    this_room.remove(&client_id);
    // remove the room if this was the last client to leave
    if this_room.is_empty() {
        state.rooms.remove(room);
    }
}


/// function to let the server know what the message is to send to others
async fn read_messages_from_client(
    connection: Connection,
    state: SharedState,
    room: &str,
    client_id: u64,
) -> Result<()> {
    loop {
        // one bidir stream per connection
        let (mut send, mut recv) = connection.accept_bi().await?;

        let mut message_bytes = Vec::new();
        // extend only a bit every time we exceed the allocated size
        let mut buffer = [0u8; 1024];

        // received bytes if we received bytes and extend the buffer if its big
        // but it cant be too big or we will send a graceful error through anyhow
        while let Some(recv_bytes) = recv.read(&mut buffer).await? {
            message_bytes.extend_from_slice(&buffer[..recv_bytes]);

            if message_bytes.len() > MAX_MESSAGE_BYTES {
                anyhow::bail!("msg is too big! {} is max size! {} was message size!", MAX_MESSAGE_BYTES, message_bytes.len());
            }
        }
        // skip u if u are empty
        if message_bytes.is_empty() {
            continue;
        }

        // decode the raw bytes into string
        // TODO: change this in the future when we arent just sending strings
        let message = String::from_utf8(message_bytes)?;

        println!("client {} in {} sent: {}", client_id, room, message);

        broadcast_to_room(&state, &room, format!("client {}: {}", client_id, message)).await;

        send.write_all(b"hi i saw ur msg hi").await?;
        send.finish().await?;
    }
}

/// sends a message to all clients in a room
async fn broadcast_to_room(state: &SharedState, room: &str, message: String) {
    let clients = {
        // oooh we are modifying the global server state againn
        let state = state.lock().await;


        // copies all the outboxes
        state
            .rooms
            .get(room)
            .map(|clients| {
                clients
                    .iter()
                    .map(|(&client_id, outbox)| (client_id, outbox.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    // note that the state is unlocked by this point so sending messages is kind of async

    // sends the message to all the outboxes
    clients.iter().for_each(|(_, outbox)| {
        if outbox.send(message.clone()).is_err() {
            eprintln!("client could not receive broadcast");
        }
    });
}
/// send a one way message to the client
async fn send_message_to_client(connection: &Connection, message: &str) -> Result<()> {
    // server sends a one way message to the client
    // browser doesnt have to answer to this
    let mut stream = connection.open_uni().await?.await?;

    stream.write_all(message.as_bytes()).await?;
    stream.finish().await?;

    Ok(()) // ok!!!
}
