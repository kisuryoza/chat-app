use std::net::SocketAddr;

use futures::{future, SinkExt};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed, FramedRead, FramedWrite};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use chat_core::{
    event::{AuthenticationStatus, RegistrationStatus},
    prelude::*,
};

use crate::{
    cli::{ask_for_command, ask_for_credentials, Cli},
    types::{Client, ThreadCommunication},
};

mod recieve;
mod send;

pub(crate) async fn handle_connection(addr: &SocketAddr) -> Result<()> {
    let tcp_stream = TcpStream::connect(addr).await.map_err(Error::io)?;
    info!("Connected to {}", addr);
    let mut stream = Framed::new(tcp_stream, BytesCodec::new());

    let (username, password) = ask_for_credentials()?;
    let mut client = Client::new(username, password);

    let shared_key =
        chat_core::key_exchange(&mut stream, client.event().clone(), client.crypto()).await?;
    info!("Shared secret with server was negotiated");
    debug!(SharedSecret = chat_core::crypto::key_to_emojies(&shared_key));
    client.set_shared_secret(shared_key);

    let cmd = ask_for_command()?;
    match cmd {
        Cli::Quit => return Ok(()),
        Cli::Login => authenticate(&mut stream, &client).await?,
        Cli::Register => register(&mut stream, &client).await?,
        _ => return Err(Error::generic("expected :login or :register")),
    }
    info!("Authenticated");

    let tcp_stream = stream.into_inner();
    let (r, w) = tcp_stream.into_split();
    let stream = FramedRead::new(r, BytesCodec::new());
    let sink = FramedWrite::new(w, BytesCodec::new());

    let (comm1, comm2) = ThreadCommunication::new();
    future::try_join(
        recieve::recieve(stream, client.clone(), comm1),
        send::send(sink, client, comm2),
    )
    .await?;
    Ok(())
}

async fn register(stream: &mut Framed<TcpStream, BytesCodec>, client: &Client) -> Result<()> {
    trace!("Initiating registration");
    let event = EventBuilder::construct(client.event().clone(), client.crypto())
        .registration_request(client.username(), client.password())
        .encrypt(client.shared_secret())?
        .then(|e| bytes::BytesMut::from(e.as_slice()));

    stream.send(event).await.map_err(Error::io)?;
    let recieved = chat_core::recieve(stream).await?;

    let deconstructed = EventBuilder::deconstruct(client.event().clone(), client.crypto())
        .decrypt(client.shared_secret(), &recieved)?;
    let deserialized = deconstructed.deserialize()?;
    let response = deserialized.expect_registration_response()?;

    let status = response.status();
    if *status != RegistrationStatus::Success {
        return Err(Error::generic(format!("Registration failure: {status}")));
    }

    Ok(())
}

async fn authenticate(stream: &mut Framed<TcpStream, BytesCodec>, client: &Client) -> Result<()> {
    trace!("Initiating authentication");
    let event = EventBuilder::construct(client.event().clone(), client.crypto())
        .authentication_request(client.username(), client.password())
        .encrypt(client.shared_secret())?
        .then(|e| bytes::BytesMut::from(e.as_slice()));

    stream.send(event).await.map_err(Error::io)?;
    let recieved = chat_core::recieve(stream).await?;

    let deconstructed = EventBuilder::deconstruct(client.event().clone(), client.crypto())
        .decrypt(client.shared_secret(), &recieved)?;
    let deserialized = deconstructed.deserialize()?;
    let response = deserialized.expect_authentication_response()?;

    let status = response.status();
    if *status != AuthenticationStatus::Success {
        return Err(Error::generic(format!("Authentication failure: {status}")));
    }

    Ok(())
}
