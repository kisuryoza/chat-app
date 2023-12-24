use capnp::serialize_packed;

use super::types;
use crate::prelude::*;

mod schema_capnp {
    #![allow(unused)]
    include!(concat!(env!("OUT_DIR"), "/schema_capnp.rs"));
}

#[derive(Default, Clone, Copy)]
pub struct Capnp;

impl From<capnp::Error> for crate::error::Error {
    fn from(value: capnp::Error) -> Self {
        Self::decode(value)
    }
}

impl From<capnp::NotInSchema> for crate::error::Error {
    fn from(value: capnp::NotInSchema) -> Self {
        Self::decode(value)
    }
}

impl EventSchema for Capnp {}

impl Serializable for Capnp {
    fn serialize(&self, entity: types::Entity) -> Vec<u8> {
        let mut message = capnp::message::Builder::new_default();
        let mut capnp_entity = message.init_root::<schema_capnp::entity::Builder>();
        capnp_entity.set_timestamp(super::timestamp());
        let mut capnp_kind = capnp_entity.init_kind();

        match entity.kind() {
            EventKind::Handshake(inner) => serialize::handshake(&mut capnp_kind, inner),
            EventKind::Registration(inner) => serialize::registration(&mut capnp_kind, inner),
            EventKind::Authentication(inner) => serialize::authentication(&mut capnp_kind, inner),
            EventKind::Message(inner) => serialize::message(&mut capnp_kind, inner),
        };

        let mut buf = Vec::new();
        let _ = serialize_packed::write_message(&mut buf, &message);
        buf
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<types::Entity> {
        use schema_capnp::entity::kind::Which;

        let message_reader =
            serialize_packed::read_message(bytes, capnp::message::ReaderOptions::new())?;
        let capnp_entity = message_reader.get_root::<schema_capnp::entity::Reader>()?;
        let timestamp = capnp_entity.get_timestamp();
        let kind = capnp_entity.get_kind().which()?;

        let kind = match kind {
            Which::Handshake(inner) => deserialize::handshake(inner?)?,
            Which::Registration(inner) => deserialize::registration(inner?)?,
            Which::Authentication(inner) => deserialize::authentication(inner?)?,
            Which::Message(inner) => deserialize::message(inner?)?,
        };

        Ok(types::Entity::new(timestamp, kind.into()))
    }
}

mod serialize {
    use super::{schema_capnp, types, Encodable};
    use schema_capnp::entity::kind::Builder;

    pub fn handshake(capnp_kind: &mut Builder<'_>, kind: &types::Handshake) {
        let mut capnp_kind = capnp_kind.reborrow().init_handshake();

        let pub_key = kind.pub_key().encode();
        capnp_kind.set_pub_key(pub_key.as_str().into());
    }

    pub fn registration(capnp_kind: &mut Builder<'_>, kind: &types::Registration<'_>) {
        let capnp_kind = capnp_kind.reborrow().init_registration().init_kind();
        match kind {
            types::Registration::Request(inner) => {
                let username = inner.username();
                let password = inner.password();
                let mut req = capnp_kind.init_request();

                req.set_username(username.into());
                req.set_password(password.into());
            }
            types::Registration::Response(inner) => {
                let status = match inner.status() {
                    types::RegistrationStatus::Success => {
                        schema_capnp::registration::response::Status::Success
                    }
                    types::RegistrationStatus::UserExists => {
                        schema_capnp::registration::response::Status::UserExists
                    }
                };
                let mut resp = capnp_kind.init_response();
                resp.set_status(status);
            }
        }
    }

    pub fn authentication(capnp_kind: &mut Builder<'_>, kind: &types::Authentication<'_>) {
        let capnp_kind = capnp_kind.reborrow().init_authentication().init_kind();
        match kind {
            types::Authentication::Request(inner) => {
                let username = inner.username();
                let password = inner.password();
                let mut req = capnp_kind.init_request();

                req.set_username(username.into());
                req.set_password(password.into());
            }
            types::Authentication::Response(inner) => {
                let status = match inner.status() {
                    types::AuthenticationStatus::Success => {
                        schema_capnp::authentication::response::Status::Success
                    }
                    types::AuthenticationStatus::UserDoesNotExist => {
                        schema_capnp::authentication::response::Status::UserDoesNotExist
                    }
                    types::AuthenticationStatus::WrongPassword => {
                        schema_capnp::authentication::response::Status::WrongPassword
                    }
                };
                let mut resp = capnp_kind.init_response();
                resp.set_status(status);
            }
        }
    }

    pub fn message(capnp_kind: &mut Builder<'_>, kind: &types::Message<'_>) {
        let mut capnp_kind = capnp_kind.reborrow().init_message();

        let sender = kind.sender();
        let text = kind.text();
        capnp_kind.set_sender(sender.into());
        capnp_kind.set_text(text.into());
    }
}

mod deserialize {
    use super::{schema_capnp, types, Encodable, Error, EventKind, PublicKey, Result, Then};

    pub fn handshake<'a>(inner: schema_capnp::handshake::Reader<'_>) -> Result<EventKind<'a>> {
        let pub_key = inner
            .get_pub_key()?
            .as_bytes()
            .then(PublicKey::try_decode)?;
        Ok(EventKind::Handshake(types::Handshake::new(pub_key)))
    }

    pub fn registration<'a>(
        inner: schema_capnp::registration::Reader<'_>,
    ) -> Result<EventKind<'a>> {
        use schema_capnp::registration::{kind::Which, response};

        let regi = match inner.get_kind().which()? {
            Which::Request(inner) => {
                let inner = inner?;
                let username = inner.get_username()?.to_string().map_err(Error::generic)?;
                let password = inner.get_password()?.to_string().map_err(Error::generic)?;
                let req = types::RegistrationRequest::new(username.into(), password.into());
                types::Registration::Request(req)
            }
            Which::Response(inner) => {
                let inner = inner?;
                let status = match inner.get_status()? {
                    response::Status::Success => types::RegistrationStatus::Success,
                    response::Status::UserExists => types::RegistrationStatus::UserExists,
                };
                let req = types::RegistrationResponse::new(status);
                types::Registration::Response(req)
            }
        };
        Ok(EventKind::Registration(regi))
    }

    pub fn authentication<'a>(
        inner: schema_capnp::authentication::Reader<'_>,
    ) -> Result<EventKind<'a>> {
        use schema_capnp::authentication::{kind::Which, response};

        let regi = match inner.get_kind().which()? {
            Which::Request(inner) => {
                let inner = inner?;
                let username = inner.get_username()?.to_string().map_err(Error::generic)?;
                let password = inner.get_password()?.to_string().map_err(Error::generic)?;
                let req = types::AuthenticationRequest::new(username.into(), password.into());
                types::Authentication::Request(req)
            }
            Which::Response(inner) => {
                let inner = inner?;
                let status = match inner.get_status()? {
                    response::Status::Success => types::AuthenticationStatus::Success,
                    response::Status::UserDoesNotExist => {
                        types::AuthenticationStatus::UserDoesNotExist
                    }
                    response::Status::WrongPassword => types::AuthenticationStatus::WrongPassword,
                };
                let req = types::AuthenticationResponse::new(status);
                types::Authentication::Response(req)
            }
        };
        Ok(EventKind::Authentication(regi))
    }

    pub fn message<'a>(inner: schema_capnp::message::Reader<'_>) -> Result<EventKind<'a>> {
        let sender = inner.get_sender()?.to_string().map_err(Error::generic)?;
        let text = inner.get_text()?.to_string().map_err(Error::generic)?;

        Ok(EventKind::Message(types::Message::new(
            sender.into(),
            text.into(),
        )))
    }
}

impl Constructable for Capnp {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handshake() {
        crate::event::tests::handshake(Capnp);
    }

    #[test]
    fn registration() {
        crate::event::tests::registration(Capnp);
    }

    #[test]
    fn authentication() {
        crate::event::tests::authentication(Capnp);
    }

    #[test]
    fn message() {
        crate::event::tests::message(Capnp);
    }
}
