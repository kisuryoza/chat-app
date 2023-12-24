Making an asynchronous chat app with E2EE for fun. Protobuf and Capnproto are used as de/serialization protocol.

Cryptography system:
- Key Exchange is [x25519](https://docs.rs/curve25519-dalek)
- Encryption is [XChaCha20-Poly1305](https://docs.rs/chacha20poly1305)
- Message Digest is [Argon2](https://docs.rs/argon2)
- Key Derivation is [BLAKE3](https://docs.rs/blake3)
