use crate::prelude::*;

mod capnp;
mod event_builder;
mod protobuf;
mod types;

pub use capnp::Capnp;
pub use event_builder::EventBuilder;
pub use protobuf::Protobuf;
pub use types::*;

pub trait EventSchema: Serializable + Constructable {}

pub trait Serializable {
    fn serialize(&self, entity: Entity<'_>) -> Vec<u8>;
    fn deserialize(&self, bytes: &[u8]) -> Result<Entity<'_>>;
}

pub trait Constructable: Serializable {
    fn construct_handshake(&self, pub_key: &PublicKey) -> types::Entity<'_> {
        let a = types::Handshake::new(*pub_key);
        let kind = types::EventKind::Handshake(a);
        types::Entity::new(timestamp(), kind.into())
    }

    fn construct_registration_request<'a>(
        &'a self,
        username: &'a str,
        password: &'a str,
    ) -> types::Entity<'a> {
        let a = types::RegistrationRequest::new(username.into(), password.into());
        let a = types::Registration::Request(a);
        let kind = types::EventKind::Registration(a);
        types::Entity::new(timestamp(), kind.into())
    }

    fn construct_registration_response(
        &self,
        status: types::RegistrationStatus,
    ) -> types::Entity<'_> {
        let a = types::RegistrationResponse::new(status);
        let a = types::Registration::Response(a);
        let kind = types::EventKind::Registration(a);
        types::Entity::new(timestamp(), kind.into())
    }

    fn construct_authentication_request<'a>(
        &'a self,
        username: &'a str,
        password: &'a str,
    ) -> types::Entity<'a> {
        let a = types::AuthenticationRequest::new(username.into(), password.into());
        let a = types::Authentication::Request(a);
        let kind = types::EventKind::Authentication(a);
        types::Entity::new(timestamp(), kind.into())
    }

    fn construct_authentication_response(
        &self,
        status: types::AuthenticationStatus,
    ) -> types::Entity<'_> {
        let a = types::AuthenticationResponse::new(status);
        let a = types::Authentication::Response(a);
        let kind = types::EventKind::Authentication(a);
        types::Entity::new(timestamp(), kind.into())
    }

    fn construct_message<'a>(&'a self, sender: &'a str, text: &'a str) -> types::Entity<'a> {
        let a = types::Message::new(sender.into(), text.into());
        let kind = types::EventKind::Message(a);
        types::Entity::new(timestamp(), kind.into())
    }
}

pub fn timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use once_cell::sync::Lazy;
    static PUB_KEY: Lazy<PublicKey> = Lazy::new(|| PublicKey::new(rand::random()));
    static AUTH_STATUS: AuthenticationStatus = AuthenticationStatus::Success;
    static REGI_STATUS: RegistrationStatus = RegistrationStatus::Success;
    static USERNAME: &str = "Badum";
    static PASSWORD: &str = "a$$word";
    static SENDER: &str = "Meme";
    static TEXT: &str = "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.";

    pub(crate) fn handshake<E: EventSchema + Clone>(event: E) {
        let entity = event.construct_handshake(&PUB_KEY);
        let serialized = event.serialize(entity);
        handle_serialized(event.clone(), &serialized).unwrap();
    }

    pub(crate) fn registration<E: EventSchema + Clone>(event: E) {
        let entity = event.construct_registration_request(USERNAME, PASSWORD);
        let serialized = event.serialize(entity);
        handle_serialized(event.clone(), &serialized).unwrap();

        let entity = event.construct_registration_response(REGI_STATUS);
        let serialized = event.serialize(entity);
        handle_serialized(event.clone(), &serialized).unwrap();
    }

    pub(crate) fn authentication<E: EventSchema + Clone>(event: E) {
        let entity = event.construct_authentication_request(USERNAME, PASSWORD);
        let serialized = event.serialize(entity);
        handle_serialized(event.clone(), &serialized).unwrap();

        let entity = event.construct_authentication_response(AUTH_STATUS);
        let serialized = event.serialize(entity);
        handle_serialized(event.clone(), &serialized).unwrap();
    }

    pub(crate) fn message<E: EventSchema + Clone>(event: E) {
        let entity = event.construct_message(SENDER, TEXT);
        let serialized = event.serialize(entity);
        handle_serialized(event.clone(), &serialized).unwrap();
    }

    fn handle_serialized<E: EventSchema + Clone>(event: E, serialized: &[u8]) -> Result<()> {
        let deserialized = event.deserialize(serialized)?;
        match deserialized.kind() {
            EventKind::Handshake(kind) => handle_handshake(kind),
            EventKind::Registration(kind) => handle_registration(kind),
            EventKind::Authentication(kind) => handle_authentication(kind),
            EventKind::Message(kind) => handle_message(kind),
        }
        Ok(())
    }

    fn handle_handshake(kind: &types::Handshake) {
        assert_eq!(*PUB_KEY, *kind.pub_key());
    }

    fn handle_registration(kind: &types::Registration<'_>) {
        match kind {
            types::Registration::Request(req) => {
                assert_eq!(USERNAME, req.username());
                assert_eq!(PASSWORD, req.password());
            }
            types::Registration::Response(resp) => {
                assert_eq!(REGI_STATUS, *resp.status());
            }
        }
    }

    fn handle_authentication(kind: &types::Authentication<'_>) {
        match kind {
            types::Authentication::Request(req) => {
                assert_eq!(USERNAME, req.username());
                assert_eq!(PASSWORD, req.password());
            }
            types::Authentication::Response(resp) => {
                assert_eq!(AUTH_STATUS, *resp.status());
            }
        }
    }

    fn handle_message(kind: &types::Message<'_>) {
        assert_eq!(SENDER, kind.sender());
        assert_eq!(TEXT, kind.text());
    }
}
