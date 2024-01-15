use chat_core::prelude::*;

#[derive(Clone)]
pub(crate) struct Client {
    username: String,
    password: String,
    event: Capnp,
    crypto: Crypto,
    /// Shared secret between this client and a server.
    /// Needs to encrypt all communicataions between the client and the server.
    server_secret: Option<SharedSecret>,
    /// Shared secret between this client and another client inside their session.
    /// Used to encrypt data between this client and the other one.
    session_secret: SessionSecret,
}

impl Client {
    pub(crate) fn new(username: String, password: String) -> Self {
        Self {
            username,
            password,
            event: Capnp::default(),
            crypto: Crypto::default(),
            server_secret: None,
            session_secret: SessionSecret::None,
        }
    }

    pub(crate) fn username(&self) -> &str {
        self.username.as_ref()
    }
    pub(crate) fn password(&self) -> &str {
        self.password.as_ref()
    }
    pub(crate) const fn event(&self) -> &Capnp {
        &self.event
    }
    pub(crate) const fn crypto(&self) -> Crypto {
        self.crypto
    }
    pub(crate) fn shared_secret(&self) -> &SharedSecret {
        self.server_secret.as_ref().unwrap()
    }
    pub(crate) fn set_shared_secret(&mut self, shared_secret: SharedSecret) {
        self.server_secret = Some(shared_secret);
    }
    pub(crate) const fn session_secret(&self) -> &SessionSecret {
        &self.session_secret
    }
    pub(crate) fn set_session_secret(&mut self, state: SessionSecret) {
        self.session_secret = state;
    }
}

#[derive(Clone)]
pub(crate) enum SessionSecret {
    None,
    PendingForShared(SecretKey),
    PendingToSend(PublicKey),
    Established(SharedSecret),
}

impl std::fmt::Display for SessionSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::PendingForShared(key) => write!(f, "PendingForSecret({})", key.encode()),
            Self::PendingToSend(key) => write!(f, "PendingToSend({})", key.encode()),
            Self::Established(key) => write!(f, "Established({})", key.encode()),
        }
    }
}

pub(crate) struct ThreadCommunication {
    pub(crate) tx: flume::Sender<SessionSecret>,
    pub(crate) rx: flume::Receiver<SessionSecret>,
}

impl ThreadCommunication {
    pub(crate) fn new() -> (Self, Self) {
        let (tx1, rx1) = flume::unbounded::<SessionSecret>();
        let (tx2, rx2) = flume::unbounded::<SessionSecret>();

        let s1 = Self { tx: tx1, rx: rx2 };
        let s2 = Self { tx: tx2, rx: rx1 };
        (s1, s2)
    }
}
