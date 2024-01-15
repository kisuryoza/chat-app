use std::borrow::Cow;

use futures::{FutureExt, SinkExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio_util::codec::{BytesCodec, FramedWrite};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use chat_core::prelude::*;

use crate::{
    cli::{self, Cli},
    types::{Client, SessionSecret, ThreadCommunication},
};

type Stream = FramedWrite<OwnedWriteHalf, BytesCodec>;

pub(crate) async fn send(
    mut sink: Stream,
    mut client: Client,
    comm: ThreadCommunication,
) -> Result<()> {
    let thread = tokio::spawn(async move {
        loop {
            match select(&mut sink, &mut client, &comm).await {
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
    let mut thread = tokio::task::spawn_blocking(cli::process_input).fuse();
    loop {
        tokio::select! {
            // This branch holds a task with stdin locked thread and can't be
            // randomly selected. For this reason the loop in this scope exists.
            Ok(input) = &mut thread =>
                break process_input(stream, client, input?, comm).await?,
            // While this branch waits for receiving data from other thread and
            // can be safely re-iterated over.
            Ok(session_secret) = comm.rx.recv_async() =>
                on_recieve_from_recieve_thread(stream, client, session_secret).await?,
        }
    }
    debug!("Event sent");
    Ok(())
}

async fn process_input(
    stream: &mut Stream,
    client: &Client,
    cli: Cli,
    comm: &ThreadCommunication,
) -> Result<()> {
    let event = match cli {
        Cli::Quit => return Err(Error::Shutdown),
        Cli::Text(text) => {
            let text = construct_text(client, text.as_ref())?;
            EventBuilder::construct(client.event().clone(), client.crypto())
                .message(client.username(), &text)
                .encrypt(client.shared_secret())?
        }
        Cli::Handshake => {
            let key = create_keys(client, comm)?;
            EventBuilder::construct(client.event().clone(), client.crypto())
                .handshake(&key)
                .encrypt(client.shared_secret())?
        }
        _ => return Err(Error::generic("expected only text, :handshake or :q")),
    };
    let event = bytes::BytesMut::from(event.as_slice());
    stream.send(event).await.map_err(Error::io)?;
    Ok(())
}

async fn on_recieve_from_recieve_thread(
    stream: &mut Stream,
    client: &mut Client,
    session_secret: SessionSecret,
) -> Result<()> {
    let event = match session_secret {
        SessionSecret::None | SessionSecret::PendingForShared(_) => unreachable!(),
        SessionSecret::PendingToSend(public_key) => {
            EventBuilder::construct(client.event().clone(), client.crypto())
                .handshake(&public_key)
                .encrypt(client.shared_secret())?
        }
        SessionSecret::Established(session_secret) => {
            client.set_session_secret(SessionSecret::Established(session_secret));
            return Ok(());
        }
    };
    let event = bytes::BytesMut::from(event.as_slice());
    stream.send(event).await.map_err(Error::io)?;
    Ok(())
}

fn construct_text<'a>(client: &'a Client, text: &'a str) -> Result<Cow<'a, str>> {
    let text: Cow<'_, str> =
        if let SessionSecret::Established(shared_secret) = client.session_secret() {
            let encrypted_text = client.crypto().encrypt(shared_secret, text.as_bytes())?;
            chat_core::crypto::base64_encode(encrypted_text).into()
        } else {
            text.into()
        };
    Ok(text)
}

fn create_keys(client: &Client, comm: &ThreadCommunication) -> Result<CryptoKey> {
    let event = match client.session_secret() {
        SessionSecret::None => {
            let (secret_key, public_key) = KeyPair::new_dh().into_split();
            let pending = SessionSecret::PendingForShared(secret_key);

            comm.tx.send(pending).map_err(Error::generic)?;
            public_key
        }
        SessionSecret::PendingForShared(_) | SessionSecret::PendingToSend(_) => unreachable!(),
        SessionSecret::Established(_) => todo!(),
    };
    Ok(event)
}
