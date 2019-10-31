# Rust Multiplayer Game
This is a project intended to prototype a multiplayer game in rust.
At this point in time, the server just echo's messages sent from the clients. Soon it will be responsible to distributing the game state to each client in game. Similarly, the clients just take command line inputs and send them to the server. More coming soon. 

# Usage
Right now I can only say that this will work in Linux.

## Running the server

The server must be running to connect to the clients

```console
./server.sh
```

## Running the host client

```console
./host.sh
```