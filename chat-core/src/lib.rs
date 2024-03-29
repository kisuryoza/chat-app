#![warn(
    clippy::all,
    // clippy::nursery,
    // clippy::pedantic,
    missing_debug_implementations,
    // missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, Framed};

pub mod crypto;
pub mod error;
pub mod event;
pub mod prelude;

use prelude::*;

const TIMEOUT_MS: u64 = 10000;

pub trait Then
where
    Self: Sized,
{
    fn then<F, O>(self, f: F) -> O
    where
        F: FnOnce(Self) -> O,
    {
        f(self)
    }
}

impl<T> Then for T where T: Sized {}

/// Waiting for [`Result::inspect_err`] to become stable.
pub trait OnErr
where
    Self: Sized,
{
    type Ok: ?Sized;
    type Err: ?Sized;

    #[must_use]
    fn on_err<F: FnOnce(&Self::Err)>(self, f: F) -> Self;
}

impl<T, E> OnErr for Result<T, E> {
    type Ok = T;
    type Err = E;

    fn on_err<F>(self, f: F) -> Self
    where
        F: FnOnce(&<Self as OnErr>::Err),
    {
        if let Err(ref val) = self {
            f(val);
        }
        self
    }
}

/// Creates and exchanges public keys, then computes shared keys.
///
/// # Errors
///
/// This function will return an error if:
/// - Failed to send a public key.
/// - Recieving is timeouted or channel is closed
/// - Recieved data is not related to handshake
pub async fn key_exchange<E, C>(
    stream: &mut Framed<TcpStream, BytesCodec>,
    event: E,
    crypto: C,
) -> Result<SharedSecret>
where
    E: EventSchema + Clone,
    C: CryptoSchema,
{
    let key_pair = KeyPair::new_dh();
    let handshake = event
        .construct_handshake(key_pair.public())
        .then(|entity| event.serialize(entity))
        .then(|event| bytes::BytesMut::from(event.as_slice()));

    stream.send(handshake).await.map_err(Error::io)?;
    let recieved = recieve(stream).await?;

    let deserialized = event.deserialize(&recieved)?;
    let sender_public = deserialized.expect_handshake()?.pub_key();

    let shared_secret = crypto.compute_dh(key_pair.secret(), sender_public);

    Ok(shared_secret)
}

pub async fn recieve(stream: &mut Framed<TcpStream, BytesCodec>) -> Result<bytes::BytesMut> {
    use std::time::Duration;
    use tokio::time::timeout;

    let recieved = timeout(Duration::from_millis(TIMEOUT_MS), stream.next())
        .await
        .map_err(|_| Error::Timeout)?;
    match recieved {
        Some(Ok(bytes)) => Ok(bytes),
        Some(Err(err)) => Err(Error::io(err)),
        None => Err(Error::io("The stream has been exhausted")),
    }
}
