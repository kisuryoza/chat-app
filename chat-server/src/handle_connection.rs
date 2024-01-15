use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use futures::SinkExt;
use tokio::{
    net::TcpStream,
    sync::{mpsc, Mutex},
};
use tokio_stream::StreamExt;
use tokio_util::codec::{BytesCodec, Framed};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use chat_core::prelude::*;

type Tx = mpsc::UnboundedSender<Vec<u8>>;
type Rx = mpsc::UnboundedReceiver<Vec<u8>>;

/// Data that is shared between all peers in the server.
///
/// This is the set of `Tx` handles for all connected clients. Whenever a
/// message is received from a client, it is broadcasted to all peers by
/// iterating over the `peers` entries and sending a copy of the message on each
/// `Tx`.
pub(crate) struct Shared {
    peers: HashMap<SocketAddr, Tx>,
}

impl Shared {
    pub(crate) fn new() -> Self {
        Self {
            peers: HashMap::new(),
        }
    }

    /// Send a message to every peer, except for the sender.
    async fn broadcast(&mut self, sender: &SocketAddr, message: &[u8]) {
        for peer in self.peers.iter_mut() {
            if peer.0 == sender {
                continue;
            }
            let _ = peer.1.send(message.into());
        }
    }
}

/// The state for each connected client.
pub(crate) struct Peer {
    stream: Framed<TcpStream, BytesCodec>,

    /// Receive half of the message channel.
    ///
    /// This is used to receive messages from peers. When a message is received
    /// off of this `Rx`, it will be written to the socket.
    rx: Rx,

    shared_key: SharedSecret,
}

impl Peer {
    async fn new(
        state: &Arc<Mutex<Shared>>,
        stream: Framed<TcpStream, BytesCodec>,
        shared_key: SharedSecret,
    ) -> Result<Self> {
        let socker_addr = stream.get_ref().peer_addr().map_err(Error::io)?;
        let (tx, rx) = mpsc::unbounded_channel();
        state.lock().await.peers.insert(socker_addr, tx);

        Ok(Self {
            stream,
            rx,
            shared_key,
        })
    }

    pub(crate) fn stream_mut(&mut self) -> &mut Framed<TcpStream, BytesCodec> {
        &mut self.stream
    }
    pub(crate) const fn shared_key(&self) -> &CryptoKey {
        &self.shared_key
    }
}

/// Process an individual client
pub(crate) async fn process(
    server: crate::types::Server,
    state: Arc<Mutex<Shared>>,
    tcp_stream: TcpStream,
    addr: SocketAddr,
) -> Result<()> {
    let mut stream = Framed::new(tcp_stream, BytesCodec::new());

    let shared_key =
        chat_core::key_exchange(&mut stream, server.event().clone(), server.crypto()).await?;
    info!("Shared secret with {} was negotiated", addr);
    debug!(
        address = addr.to_string(),
        SharedSecret = chat_core::crypto::key_to_emojies(&shared_key)
    );

    let mut peer = Peer::new(&state, stream, shared_key).await?;

    crate::authentication::main(&server, &mut peer).await?;
    info!("{} authenticated", addr);

    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            // A message was received from some peer. Send it to the current peer.
            Some(msg) = peer.rx.recv() => {
                let msg = CryptoSchema::encrypt(&server.crypto(), &peer.shared_key, &msg)?
                    .then(|e| bytes::BytesMut::from(e.as_slice()));
                peer.stream.send(msg).await.map_err(Error::io)?;
            }
            result = peer.stream.next() => match result {
                // A message was received from the current peer.
                Some(Ok(bytes)) => {
                    if let Err(err) = on_recieve_from_curr_peer(&server, &state, &mut peer, bytes).await {
                        warn!("error occurred while working with recieved data for {}; error = {}", addr, err);
                    }
                },
                // An error occurred.
                Some(Err(err)) => {
                    warn!("error occurred while processing income for {}; error = {:?}", addr, err);
                }
                // The stream has been exhausted.
                None => break,
            },
        }
    }

    // If this section is reached it means that the client was disconnected!
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);
    }

    Ok(())
}

async fn on_recieve_from_curr_peer(
    server: &crate::types::Server,
    state: &Arc<Mutex<Shared>>,
    peer: &mut Peer,
    recieved: bytes::BytesMut,
) -> Result<()> {
    let event = server.event();
    let decrypted = server.crypto().decrypt(peer.shared_key(), &recieved)?;
    let deserialized = event.deserialize(&decrypted)?;

    match deserialized.kind() {
        EventKind::Registration(_) | EventKind::Authentication(_) => (),
        EventKind::Message(_) | EventKind::Handshake(_) => {
            let socker_addr = peer.stream_mut().get_ref().peer_addr().map_err(Error::io)?;
            state.lock().await.broadcast(&socker_addr, &decrypted).await;
        }
    }

    Ok(())
}
