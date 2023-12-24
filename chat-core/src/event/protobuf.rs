use super::types;
use crate::prelude::*;

mod _protobuf {
    include!(concat!(env!("OUT_DIR"), "/protobuf_schema.rs"));
}

#[derive(Default, Clone, Copy)]
pub struct Protobuf;

impl EventSchema for Protobuf {}

impl Serializable for Protobuf {
    fn serialize(&self, entity: types::Entity) -> Vec<u8> {
        use prost::Message;

        let kind = match entity.kind() {
            EventKind::Handshake(kind) => serialize::handshake(kind),
            EventKind::Registration(kind) => serialize::registration(kind),
            EventKind::Authentication(kind) => serialize::authentication(kind),
            EventKind::Message(kind) => serialize::message(kind),
        };

        let entity = _protobuf::Entity {
            timestamp: super::timestamp(),
            kind: Some(kind),
        };

        let mut buf = Vec::with_capacity(entity.encoded_len());
        // Unwrap is safe, since we have reserved sufficient capacity in the vector.
        entity.encode(&mut buf).unwrap();
        buf
    }

    fn deserialize(&self, bytes: &[u8]) -> Result<types::Entity> {
        use _protobuf::entity::Kind;
        use prost::Message;
        use std::io::Cursor;

        let decoded = _protobuf::Entity::decode(&mut Cursor::new(bytes)).map_err(Error::decode)?;
        let timestamp = decoded.timestamp;
        let kind = decoded
            .kind
            .ok_or_else(|| Error::decode("Bad event structure"))?;

        let kind = match kind {
            Kind::Handshake(kind) => deserialize::handshake(kind)?,
            Kind::Registration(kind) => deserialize::registration(kind)?,
            Kind::Authentication(kind) => deserialize::authentication(kind)?,
            Kind::Message(kind) => deserialize::message(kind),
        };

        let entity = types::Entity::new(timestamp, kind.into());
        Ok(entity)
    }
}

mod serialize {
    use super::{Encodable, _protobuf, types};
    use _protobuf::entity::Kind;

    pub fn handshake(kind: &types::Handshake) -> Kind {
        let a = _protobuf::Handshake {
            pub_key: kind.pub_key().encode(),
        };
        Kind::Handshake(a)
    }

    pub fn registration(kind: &types::Registration<'_>) -> Kind {
        let kind = match kind {
            types::Registration::Request(inner) => {
                let req = _protobuf::registration::Request {
                    username: inner.username().to_owned(),
                    password: inner.password().to_owned(),
                };
                _protobuf::registration::Kind::Request(req)
            }
            types::Registration::Response(inner) => {
                let req = _protobuf::registration::Response {
                    status: *inner.status() as i32,
                };
                _protobuf::registration::Kind::Response(req)
            }
        };
        let a = _protobuf::Registration { kind: Some(kind) };
        Kind::Registration(a)
    }

    pub fn authentication(kind: &types::Authentication<'_>) -> Kind {
        let kind = match kind {
            types::Authentication::Request(inner) => {
                let req = _protobuf::authentication::Request {
                    username: inner.username().to_owned(),
                    password: inner.password().to_owned(),
                };
                _protobuf::authentication::Kind::Request(req)
            }
            types::Authentication::Response(inner) => {
                let req = _protobuf::authentication::Response {
                    status: *inner.status() as i32,
                };
                _protobuf::authentication::Kind::Response(req)
            }
        };
        let a = _protobuf::Authentication { kind: Some(kind) };
        Kind::Authentication(a)
    }

    pub fn message(kind: &types::Message<'_>) -> Kind {
        let a = _protobuf::Message {
            sender: kind.sender().to_owned(),
            text: kind.text().to_owned(),
        };
        Kind::Message(a)
    }
}

mod deserialize {
    use super::{Encodable, Error, EventKind, PublicKey, Result, Then, _protobuf, types};

    pub fn handshake<'a>(kind: _protobuf::Handshake) -> Result<EventKind<'a>> {
        let pub_key = kind.pub_key.then(PublicKey::try_decode)?;
        Ok(EventKind::Handshake(types::Handshake::new(pub_key)))
    }

    pub fn registration<'a>(kind: _protobuf::Registration) -> Result<EventKind<'a>> {
        let kind = kind
            .kind
            .ok_or_else(|| Error::decode("Bad event structure"))?;

        let a = match kind {
            _protobuf::registration::Kind::Request(req) => {
                let request =
                    types::RegistrationRequest::new(req.username.into(), req.password.into());
                types::Registration::Request(request)
            }
            _protobuf::registration::Kind::Response(resp) => {
                let status = types::RegistrationStatus::try_from(resp.status)?;
                let response = types::RegistrationResponse::new(status);
                types::Registration::Response(response)
            }
        };

        Ok(EventKind::Registration(a))
    }

    pub fn authentication<'a>(kind: _protobuf::Authentication) -> Result<EventKind<'a>> {
        let kind = kind
            .kind
            .ok_or_else(|| Error::decode("Bad event structure"))?;

        let a = match kind {
            _protobuf::authentication::Kind::Request(req) => {
                let request =
                    types::AuthenticationRequest::new(req.username.into(), req.password.into());
                types::Authentication::Request(request)
            }
            _protobuf::authentication::Kind::Response(resp) => {
                let status = types::AuthenticationStatus::try_from(resp.status)?;
                let response = types::AuthenticationResponse::new(status);
                types::Authentication::Response(response)
            }
        };

        Ok(EventKind::Authentication(a))
    }

    pub fn message<'a>(kind: _protobuf::Message) -> EventKind<'a> {
        let sender = kind.sender;
        let text = kind.text;
        EventKind::Message(types::Message::new(sender.into(), text.into()))
    }
}

impl Constructable for Protobuf {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handshake() {
        crate::event::tests::handshake(Protobuf);
    }

    #[test]
    fn registration() {
        crate::event::tests::registration(Protobuf);
    }

    #[test]
    fn authentication() {
        crate::event::tests::authentication(Protobuf);
    }

    #[test]
    fn message() {
        crate::event::tests::message(Protobuf);
    }
}
