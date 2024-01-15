use chat_core::prelude::*;

#[derive(Clone)]
pub(crate) struct Server {
    event: Capnp,
    crypto: Crypto,
    db_pool: sqlx::PgPool,
}

impl Server {
    pub(crate) fn new(db_pool: sqlx::PgPool) -> Self {
        Self {
            event: Capnp::default(),
            crypto: Crypto::default(),
            db_pool,
        }
    }

    pub(crate) const fn event(&self) -> &Capnp {
        &self.event
    }
    pub(crate) const fn crypto(&self) -> Crypto {
        self.crypto
    }
    pub(crate) const fn db_pool(&self) -> &sqlx::PgPool {
        &self.db_pool
    }
}
