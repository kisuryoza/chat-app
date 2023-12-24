use chat_core::prelude::*;

#[derive(Clone)]
pub struct Server {
    event: Capnp,
    crypto: Crypto,
    db_pool: sqlx::PgPool,
}

impl Server {
    pub const fn new(db_pool: sqlx::PgPool) -> Self {
        Self {
            event: Capnp,
            crypto: Crypto,
            db_pool,
        }
    }

    pub const fn event(&self) -> Capnp {
        self.event
    }
    pub const fn crypto(&self) -> Crypto {
        self.crypto
    }
    pub const fn db_pool(&self) -> &sqlx::PgPool {
        &self.db_pool
    }
}
