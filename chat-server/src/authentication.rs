use futures::SinkExt;
use tokio_stream::StreamExt;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

use chat_core::{
    event::{AuthenticationStatus, RegistrationStatus},
    prelude::*,
};

use crate::handle_connection::Peer;

pub(crate) async fn main(server: &crate::types::Server, peer: &mut Peer) -> Result<()> {
    let socker_addr = peer.stream_mut().get_ref().peer_addr().map_err(Error::io)?;
    let recieved = match peer.stream_mut().next().await {
        Some(Ok(bytes)) => bytes,
        Some(Err(err)) => {
            warn!(
                "error occurred while processing income for {}; error = {:?}",
                socker_addr, err
            );
            return Err(Error::io(err));
        }
        // The stream has been exhausted.
        None => return Ok(()),
    };

    let event = server.event();
    let crypto = server.crypto();
    let decrypted = crypto.decrypt(peer.shared_key(), &recieved)?;
    let deserialized = event.deserialize(&decrypted)?;

    match deserialized.kind() {
        EventKind::Registration(regi) => {
            trace!("Processing RegistrationRequest");
            let req = match regi {
                chat_core::event::Registration::Request(inner) => inner,
                chat_core::event::Registration::Response(_) => {
                    return Err(Error::generic(
                        "Was expecting client to participate in authentication processs",
                    ))
                }
            };
            let status = register(server, req.username(), req.password()).await?;

            let event = EventBuilder::construct(server.event().clone(), crypto)
                .registration_response(status)
                .encrypt(peer.shared_key())?
                .then(|e| bytes::BytesMut::from(e.as_slice()));

            peer.stream_mut().send(event).await.map_err(Error::io)?;

            if status != RegistrationStatus::Success {
                return Err(Error::generic(format!("Registration failure: {status}")));
            }
        }
        EventKind::Authentication(auth) => {
            trace!("Processing AuthenticationRequest");
            let req = match auth {
                chat_core::event::Authentication::Request(inner) => inner,
                chat_core::event::Authentication::Response(_) => {
                    return Err(Error::generic(
                        "Was expecting client to participate in authentication processs",
                    ))
                }
            };
            let status = authenticate(server, req.username(), req.password()).await?;

            let event = EventBuilder::construct(server.event().clone(), crypto)
                .authentication_response(status)
                .encrypt(peer.shared_key())?
                .then(|e| bytes::BytesMut::from(e.as_slice()));

            peer.stream_mut().send(event).await.map_err(Error::io)?;

            if status != AuthenticationStatus::Success {
                return Err(Error::generic(format!("Authentication failure: {status}")));
            }
        }
        _ => {
            return Err(Error::generic(
                "Was expecting client to participate in authentication processs",
            ))
        }
    }

    Ok(())
}

async fn register(
    server: &crate::types::Server,
    login: &str,
    password: &str,
) -> Result<RegistrationStatus> {
    let password = server.crypto().hash_password(password.as_bytes())?;
    let row = sqlx::query!(
        "INSERT INTO accounts ( login, password ) VALUES ( $1, $2 )",
        login,
        password
    )
    .execute(server.db_pool())
    .await;

    if row.is_err() {
        return Ok(RegistrationStatus::UserExists);
    }

    Ok(RegistrationStatus::Success)
}

async fn authenticate(
    server: &crate::types::Server,
    login: &str,
    password: &str,
) -> Result<AuthenticationStatus> {
    let row = sqlx::query!(
        "SELECT login, password FROM accounts WHERE login = $1",
        login
    )
    .fetch_one(server.db_pool())
    .await;

    if row.is_err() {
        return Ok(AuthenticationStatus::UserDoesNotExist);
    }
    let row = row.unwrap();

    let verified = server
        .crypto()
        .verify_password(&row.password, password.as_bytes())?;
    if !verified {
        return Ok(AuthenticationStatus::WrongPassword);
    }

    Ok(AuthenticationStatus::Success)
}
