use crate::{
    event::types::{AuthenticationStatus, Entity, RegistrationStatus},
    prelude::*,
};

#[derive(Debug)]
pub struct EventBuilder;

impl EventBuilder {
    pub const fn construct<E, C>(event_system: E, crypto_system: C) -> Builder<Constructing<E>, C>
    where
        E: EventSchema,
        C: CryptoSchema,
    {
        Builder {
            state: Constructing(event_system),
            crypto_system,
        }
    }

    pub const fn deconstruct<E, C>(
        event_system: E,
        crypto_system: C,
    ) -> Builder<Deconstructing<E>, C>
    where
        E: EventSchema,
        C: CryptoSchema,
    {
        Builder {
            state: Deconstructing(event_system),
            crypto_system,
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Constructing<E: EventSchema>(E);
#[derive(Debug)]
pub struct Constructed {
    bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct Deconstructing<E: EventSchema>(E);
#[derive(Debug)]
pub struct Decrypted<E: EventSchema> {
    event: E,
    bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct Builder<E, C> {
    state: E,
    crypto_system: C,
}

macro_rules! create_builder {
    ($self:ident, $state:ident) => {
        Builder {
            state: $state,
            crypto_system: $self.crypto_system,
        }
    };
}

///////////////////////////////////////////////////////////////////////////////
// Construction process

impl<E, C> Builder<Constructing<E>, C>
where
    E: EventSchema,
    C: CryptoSchema,
{
    pub fn handshake(self, pub_key: &PublicKey) -> Builder<Constructed, C> {
        let event = self.state.0;
        let entity = event.construct_handshake(pub_key);
        let state = Constructed {
            bytes: event.serialize(entity),
        };
        create_builder!(self, state)
    }

    pub fn registration_request(self, username: &str, password: &str) -> Builder<Constructed, C> {
        let event = self.state.0;
        let entity = event.construct_registration_request(username, password);
        let state = Constructed {
            bytes: event.serialize(entity),
        };
        create_builder!(self, state)
    }

    pub fn registration_response(self, status: RegistrationStatus) -> Builder<Constructed, C> {
        let event = self.state.0;
        let entity = event.construct_registration_response(status);
        let state = Constructed {
            bytes: event.serialize(entity),
        };
        create_builder!(self, state)
    }

    pub fn authentication_request(self, username: &str, password: &str) -> Builder<Constructed, C> {
        let event = self.state.0;
        let entity = event.construct_authentication_request(username, password);
        let state = Constructed {
            bytes: event.serialize(entity),
        };
        create_builder!(self, state)
    }
    pub fn authentication_response(self, status: AuthenticationStatus) -> Builder<Constructed, C> {
        let event = self.state.0;
        let entity = event.construct_authentication_response(status);
        let state = Constructed {
            bytes: event.serialize(entity),
        };
        create_builder!(self, state)
    }

    pub fn message(self, sender: &str, text: &str) -> Builder<Constructed, C> {
        let event = self.state.0;
        let entity = event.construct_message(sender, text);
        let state = Constructed {
            bytes: event.serialize(entity),
        };
        create_builder!(self, state)
    }
}

impl<C> Builder<Constructed, C>
where
    C: CryptoSchema,
{
    pub fn encrypt(self, key: &SecretKey) -> Result<Vec<u8>> {
        self.crypto_system.encrypt(key, &self.state.bytes)
    }
}

///////////////////////////////////////////////////////////////////////////////
// Deconstruction process

impl<E, C> Builder<Deconstructing<E>, C>
where
    E: EventSchema,
    C: CryptoSchema,
{
    pub fn decrypt(self, key: &SecretKey, blob: &[u8]) -> Result<Builder<Decrypted<E>, C>> {
        // let bytes = Some(self.crypto_system.decrypt(key, blob)?);
        let state = Decrypted {
            event: self.state.0,
            bytes: self.crypto_system.decrypt(key, blob)?,
        };
        Ok(create_builder!(self, state))
    }
}

impl<E, C> Builder<Decrypted<E>, C>
where
    E: EventSchema,
    C: CryptoSchema,
{
    pub fn deserialize(&self) -> Result<Entity<'_>> {
        let event = &self.state.event;
        event.deserialize(&self.state.bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types;

    const fn event_system() -> Protobuf {
        Protobuf
    }

    use once_cell::sync::Lazy;
    static PUB_KEY: Lazy<PublicKey> = Lazy::new(|| PublicKey::new(rand::random()));
    static AUTH_STATUS: AuthenticationStatus = AuthenticationStatus::Success;
    static REGI_STATUS: RegistrationStatus = RegistrationStatus::Success;
    static USERNAME: &str = "Badum";
    static PASSWORD: &str = "a$$word";
    static SENDER: &str = "Meme";
    static TEXT: &str = "Lorem ipsum dolor sit amet, qui minim labore adipisicing minim sint cillum sint consectetur cupidatat.";

    #[test]
    fn build_handshake() -> Result<()> {
        let constructed = EventBuilder::construct(event_system(), Crypto)
            .handshake(&PUB_KEY)
            .encrypt(&PUB_KEY)?;

        let binding =
            EventBuilder::deconstruct(event_system(), Crypto).decrypt(&PUB_KEY, &constructed)?;
        let deconstructed = binding.deserialize()?;
        handle_deconstructed(deconstructed);
        Ok(())
    }

    #[test]
    fn build_registration() -> Result<()> {
        let constructed = EventBuilder::construct(event_system(), Crypto)
            .registration_request(USERNAME, PASSWORD)
            .encrypt(&PUB_KEY)?;

        let binding =
            EventBuilder::deconstruct(event_system(), Crypto).decrypt(&PUB_KEY, &constructed)?;
        let deconstructed = binding.deserialize()?;
        handle_deconstructed(deconstructed);

        let constructed = EventBuilder::construct(event_system(), Crypto)
            .registration_response(REGI_STATUS)
            .encrypt(&PUB_KEY)?;

        let binding =
            EventBuilder::deconstruct(event_system(), Crypto).decrypt(&PUB_KEY, &constructed)?;
        let deconstructed = binding.deserialize()?;
        handle_deconstructed(deconstructed);

        Ok(())
    }

    #[test]
    fn build_authentication() -> Result<()> {
        let constructed = EventBuilder::construct(event_system(), Crypto)
            .authentication_request(USERNAME, PASSWORD)
            .encrypt(&PUB_KEY)?;

        let binding =
            EventBuilder::deconstruct(event_system(), Crypto).decrypt(&PUB_KEY, &constructed)?;
        let deconstructed = binding.deserialize()?;
        handle_deconstructed(deconstructed);

        let constructed = EventBuilder::construct(event_system(), Crypto)
            .authentication_response(AUTH_STATUS)
            .encrypt(&PUB_KEY)?;

        let binding =
            EventBuilder::deconstruct(event_system(), Crypto).decrypt(&PUB_KEY, &constructed)?;
        let deconstructed = binding.deserialize()?;
        handle_deconstructed(deconstructed);

        Ok(())
    }

    #[test]
    fn build_message() -> Result<()> {
        let constructed = EventBuilder::construct(event_system(), Crypto)
            .message(SENDER, TEXT)
            .encrypt(&PUB_KEY)?;

        let binding =
            EventBuilder::deconstruct(event_system(), Crypto).decrypt(&PUB_KEY, &constructed)?;
        let deconstructed = binding.deserialize()?;
        handle_deconstructed(deconstructed);
        Ok(())
    }

    fn handle_deconstructed(deconstructed: Entity<'_>) {
        match deconstructed.kind() {
            EventKind::Handshake(kind) => handle_handshake(kind),
            EventKind::Registration(kind) => handle_registration(kind),
            EventKind::Authentication(kind) => handle_authentication(kind),
            EventKind::Message(kind) => handle_message(kind),
        }
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
