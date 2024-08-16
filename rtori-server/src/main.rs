//! A chat server that broadcasts a message to all connections.
//!
//! This is a simple line-based server which accepts WebSocket connections,
//! reads lines from those connections, and broadcasts the lines to all other
//! connected clients.
//!
//! You can test this out by running:
//!
//!     cargo run --example server 127.0.0.1:12345
//!
//! And then in another window run:
//!
//!     cargo run --example client ws://127.0.0.1:12345/
//!
//! You can run the second command in multiple windows and then chat between the
//! two, seeing the messages from the other client as they're received. For all
//! connected clients they'll all join the same room and see everyone else's
//! messages.

use std::{
    collections::HashMap,
    env,
    io::{Error as IoError, Stdout},
    net::SocketAddr,
    sync::{mpsc::TrySendError, Arc, Mutex},
};
mod serialized;

use futures::{
    channel::mpsc::{channel, unbounded, Sender, UnboundedSender},
    SinkExt,
};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::{http::StatusCode, protocol::Message};

type Tx = Sender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Simulator,
    Consumer,
}

impl NodeKind {
    pub fn parse(s: &str) -> Option<NodeKind> {
        match () {
            _ if s.eq_ignore_ascii_case("simulator") => Some(Self::Simulator),
            _ if s.eq_ignore_ascii_case("consumer") => Some(Self::Consumer),
            _ => None,
        }
    }
}

async fn handle_simulator(
    ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    peer_map: PeerMap,
    addr: SocketAddr,
) {
    let (tx, rx) = unbounded();

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        //println!("Received a message from {}: length {}", addr, msg.to_text().unwrap().len());
        match &msg {
            tokio_tungstenite::tungstenite::Message::Text(text) => {
                let mut peers = peer_map.lock().unwrap();

                // We want to broadcast the message to everyone except ourselves.
                let broadcast_recipients = peers
                    .iter_mut()
                    .filter(|(peer_addr, _)| peer_addr != &&addr)
                    .map(|(_, ws_sink)| ws_sink);

                for recp in broadcast_recipients {
                    //println!("Broadcasting");
                    match recp.try_send(msg.clone()) {
                        Ok(()) => (),
                        Err(e) if e.is_full() => print!("-"),
                        Err(e) => panic!("Got an error while sending: {e}"),
                    }
                }
            }
            tokio_tungstenite::tungstenite::Message::Binary(data) => {
                println!("ERROR: msg is binary");
            }
            tokio_tungstenite::tungstenite::Message::Ping(ping) => {
                tx.unbounded_send(tokio_tungstenite::tungstenite::Message::Pong(
                    msg.into_data(),
                ))
                .unwrap();
            }
            _ => println!("ERROR: unsupported message type"),
        }

        future::ok(())
    });

    let send_future = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, send_future);
    future::select(broadcast_incoming, send_future).await;
}

async fn handle_consumer(
    ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    peer_map: PeerMap,
    addr: SocketAddr,
) {
    // Insert the write part of this peer to the peer map.
    let (tx, rx) = channel(1);
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let receive_from_others = rx
        .map(|m| {
            print!(".");
            <Stdout as std::io::Write>::flush(&mut std::io::stdout()).unwrap();
            Ok(m)
        })
        .forward(outgoing);

    let orders = incoming.try_for_each(|_| {
        // TODO
        future::ok(())
    });

    pin_mut!(orders, receive_from_others);
    future::select(orders, receive_from_others).await;

    peer_map.lock().unwrap().remove(&addr);
    //println!("Received a message from {}: length {}", addr, msg.to_text().unwrap().len());
}

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let mut mode: Option<NodeKind> = None;

    let ws_stream_res = tokio_tungstenite::accept_hdr_async(raw_stream, {
        let mode = &mut mode;
        move |request: &tokio_tungstenite::tungstenite::handshake::server::Request, response| {
            let uri = request.uri();
            let candidate = uri.path().trim_matches(['/', ' ']);
            let node_kind = match NodeKind::parse(candidate) {
                Some(kind) => kind,
                None => {
                    let mut resp =
                        tokio_tungstenite::tungstenite::handshake::server::ErrorResponse::default();
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                    *resp.body_mut() = Some(format!(
                        "No such kind '{candidate}', only 'simulator' and 'consumer' allowed"
                    ));
                    return Err(resp);
                }
            };
            *mode = Some(node_kind);

            Ok(response)
        }
    })
    .await;

    let ws_stream = match ws_stream_res {
        Err(tokio_tungstenite::tungstenite::Error::Http(http))
            if matches!(
                http.status(),
                tokio_tungstenite::tungstenite::http::StatusCode::NOT_FOUND
            ) =>
        {
            return
        }
        Err(e) => panic!("got error {e:?}"),
        Ok(stream) => stream,
    };
    let mode = mode.unwrap();

    println!("WebSocket connection established ({mode:?}): {}", addr);

    match mode {
        NodeKind::Simulator => handle_simulator(ws_stream, peer_map, addr).await,
        NodeKind::Consumer => handle_consumer(ws_stream, peer_map, addr).await,
    }

    println!("{} ({mode:?}) disconnected", &addr);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8081".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}
