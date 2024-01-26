simplechat
==========

A very simple chat program written in asynchronous Rust.


## Running

Currently the server only runs on localhost using port 3000 and can be started
with:

    cargo run -p simplechat-server

Once the server is running clients can be connected with:

    cargo run -p simplechat-client -- --name "John Smith"

Ctrl-C will exit the client or server.
