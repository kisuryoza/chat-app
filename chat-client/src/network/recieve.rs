use std::borrow::Cow;

use futures::StreamExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio_util::codec::{BytesCodec, FramedRead};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use chat_core::prelude::*;

use crate::types::{Client, SessionSecret, ThreadCommunication};

type Stream = FramedRead<OwnedReadHalf, BytesCodec>;

pub(crate) async fn recieve(
    mut stream: Stream,
    mut client: Client,
    comm: ThreadCommunication,
) -> Result<()> {
    let thread = tokio::spawn(async move {
        loop {
            match select(&mut stream, &mut client, &comm).await {
                Ok(()) => (),
                Err(Error::Shutdown) => return Ok(()),
                Err(err) => warn!("{}", err),
            }
        }
    });
    thread.await.map_err(Error::io)?
}

async fn select(
    stream: &mut Stream,
    client: &mut Client,
    comm: &ThreadCommunication,
) -> Result<()> {
    tokio::select! {
        recieved = stream.next() => match recieved {
            Some(Ok(bytes)) => process_recieved(&bytes, client, comm),
            Some(Err(err)) => Err(Error::io(err)),
            // The stream has been exhausted.
            None => Err(Error::Shutdown),
        },
        Ok(session_secret) = comm.rx.recv_async() => {
            debug!(SessionSecret = session_secret.to_string());
            client.set_session_secret(session_secret);
            Ok(())
        }
    }
}

fn process_recieved(
    recieved: &[u8],
    client: &mut Client,
    comm: &ThreadCommunication,
) -> Result<()> {
    let deconstructed = EventBuilder::deconstruct(client.event().clone(), client.crypto())
        .decrypt(client.shared_secret(), recieved)?;
    let deserialized = deconstructed.deserialize()?;

    let timestamp = deserialized.timestamp();
    match deserialized.kind() {
        EventKind::Handshake(kind) => process_handshake(client, comm, *kind.pub_key())?,
        EventKind::Registration(_) => todo!(),
        EventKind::Authentication(_) => todo!(),
        EventKind::Message(kind) => process_message(client, *timestamp, kind)?,
    }

    Ok(())
}

fn process_handshake(
    client: &mut Client,
    comm: &ThreadCommunication,
    public_key: PublicKey,
) -> Result<()> {
    info!("Another client wants to contribute to a new encryption key for this session");
    match client.session_secret() {
        SessionSecret::None => {
            let (my_sec, my_pub) = KeyPair::new_dh().into_split();
            let pending = SessionSecret::PendingToSend(my_pub);
            comm.tx.send(pending).map_err(Error::generic)?;

            let established = make_established_and_share(client, comm, &my_sec, &public_key)?;
            client.set_session_secret(established);
        }
        SessionSecret::PendingForShared(secret_key) => {
            let established = make_established_and_share(client, comm, secret_key, &public_key)?;
            client.set_session_secret(established);
        }
        SessionSecret::PendingToSend(_) => unreachable!(),
        SessionSecret::Established(_) => todo!(),
    }

    Ok(())
}

fn make_established_and_share(
    client: &Client,
    comm: &ThreadCommunication,
    secret_key: &SecretKey,
    public_key: &PublicKey,
) -> Result<SessionSecret> {
    let shared_secret = client.crypto().compute_dh(secret_key, public_key);
    info!("Shared secret for this session was negotiated");
    debug!(SessionSecret = chat_core::crypto::key_to_emojies(&shared_secret));

    let established = SessionSecret::Established(shared_secret);
    comm.tx.send(established.clone()).map_err(Error::generic)?;
    Ok(established)
}

fn process_message(
    client: &Client,
    timestamp: i64,
    event: &chat_core::event::Message<'_>,
) -> Result<()> {
    let text: Cow<'_, str> =
        if let SessionSecret::Established(shared_secret) = client.session_secret() {
            let decoded = chat_core::crypto::base64_decode(event.text())?;
            let decrypted_text = client.crypto().decrypt(shared_secret, &decoded)?;
            String::from_utf8(decrypted_text)
                .map_err(Error::generic)?
                .into()
        } else {
            event.text().into()
        };

    let timestamp = from_timestamp(timestamp)?;
    println!("{}: {}: {}", timestamp, event.sender(), text);
    Ok(())
}

fn from_timestamp(timestamp: i64) -> Result<String> {
    use chrono::prelude::*;

    let time = DateTime::from_timestamp(timestamp, 0)
        .ok_or_else(|| Error::generic("Timestamp is out of range"))?;
    Ok(format!("{}", time.format("%H:%M")))
}
