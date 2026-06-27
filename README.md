# burvy's portfolio site
A site to showcase a bunch of different tools in Rust
# WebTransport Server
I used the [wtransport](https://docs.rs/wtransport/latest/wtransport/) crate.
The first basic iteration of the server worked in a funky way. 
This is the version of the server as of 6/27/2026.

As of this date, the server is used to facilitate a chat application;
which allows the transmission of strings to a server and from the server to all connected clients in a room.
Notice in the server folder, there is a `ServerState`, which stores a client counter and a hash map of rooms.

Every time a client joins, `join_room` increments the client counter and assigns that id to the joined client. 
This helps differentiate between the different clients. 
Unfortunately, the counter just keeps increasing, so new clients will have pretty huge ids 
if the server is kept alive for a long time.
this isn't too much of an issue at the moment, 
but I want to add a username/password - database system eventually.

Every time a client leaves, the client is removed based on the client_id attached to them, 
and removes the room entirely if it is empty. 
Note that this might be an issue and should be looked at.

Also note, that the functions ultimately all modify an Arc-Mutex ServerState - 
it is referred to by `SharedState`, which is why it appears throughout the code 
rather than `Arc<Mutex<ServerState>>`.

Every time the `SharedState` is modified, it needs to be locked within the function.

For actually reading messages from the client, we have a convenient `read_messages_from_client` function 
that accepts a bidirectional stream from clients, from which they send a bunch of bytes over, 
4096 bytes maximum, as configured.

The bidirectional stream is technically not THAT important, 
but just to make use of it i send back the "hi i saw ur message hi" ACK. 
Later on i want to use this to send game data between server and clients.

For sending messages back, its best to return back to thinking of the system like broadcasting,
one client sends a message to the server, then the server sends the message to everyone.
This is done through a unidirectional stream. Clients do not actually have to send anything
back, they just take whatever server gives them and they do all the error handling on their end

Message history is also present, Room has a VecDeque, and now see here i used a queue structure because
i want the first messages to be sent first and in order. The way message history works is the server
sends a huge batch of strings for the client to parse, and to maintain the illusion of being in order
the messages must be sent in order.

If I used a LIFO structure like a Vec, it would look like the oldest messages were the most recent,
which is the opposite of what I want. Also, using a FIFO structure also makes this faster.
