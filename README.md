# burvy's portfolio site
A site to showcase a bunch of different tools in Rust
# WebTransport Server
I used the [wtransport](https://docs.rs/wtransport/latest/wtransport/) crate.
The first basic iteration of the server worked in a funky way. This is the version of the server as of 6/27/2026.
As of this date, the server is used to facilitate a chat application, which allows the transmission of strings to a server and from the server to all connected clients in a room.
Notice in the server folder, there is a `ServerState`, which stores a client counter and a hash map of rooms.
Every time a client joins, `join_room` increments the client counter and assigns that id to the joined client. This helps differentiate between the different clients. Unfortunately, the counter just keeps increasing, so new clients will have pretty huge ids if the server is kept alive for a long time.
