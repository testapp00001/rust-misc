# Module 10: Data in Transit

Protecting data as it moves across networks. TLS, certificate validation,
replay defenses, and traffic analysis resistance.

## Why Data in Transit Matters

Encryption at rest protects data on disk, but data spends most of its life
moving between services, clients, and servers. An attacker on the network path
can read, modify, or replay unprotected traffic. TLS (Transport Layer Security)
is the standard defense, but using it correctly requires more than just flipping
a switch.

**TLS provides three guarantees:**
- **Confidentiality** — eavesdroppers cannot read the data
- **Integrity** — tampering is detected
- **Authenticity** — you know who you are talking to

Getting any one of these wrong breaks the security model.

## TLS 1.3 vs TLS 1.2

| Feature | TLS 1.2 | TLS 1.3 |
|---------|---------|---------|
| Handshake round-trips | 2 RTT | 1 RTT (0-RTT resumption) |
| Key exchange | RSA, DH, ECDH | ECDHE only (forward secrecy mandatory) |
| Cipher suites | ~300+ | 5 (all AEAD) |
| Legacy ciphers | RC4, 3DES, CBC | Removed entirely |
| Handshake encryption | No | Yes (protects certificates) |
| 0-RTT | No | Yes (with replay risk) |

TLS 1.3 removes dangerous options. You cannot accidentally configure a weak
cipher suite because none exist. TLS 1.2 requires careful configuration to
avoid downgrade attacks and weak ciphers.

## Certificate Validation is Critical

The most common TLS mistake is disabling certificate validation:

```rust
// NEVER DO THIS
danger_accept_invalid_certs(true)
```

This makes encryption useless — you are talking to *someone* encrypted, but
you have no idea who. A man-in-the-middle can present their own certificate
and decrypt everything.

**Certificate validation checks:**
1. Is the certificate signed by a trusted CA?
2. Has the certificate expired?
3. Does the hostname match?
4. Has the certificate been revoked?

## Certificate Pinning

Standard validation trusts the entire CA system — hundreds of CAs, any of which
can issue a certificate for any domain. Certificate pinning restricts trust to
specific certificates or public keys you expect.

**Use cases:**
- Mobile apps communicating with your API
- Embedded devices with known server certificates
- High-security applications reducing CA trust surface

## Replay Attacks

An attacker records a valid message and re-sends it later. If the message is
"transfer $100", replaying it 100 times is devastating. Defenses include:
- Nonces (unique message IDs)
- Timestamps with tight windows
- Sequence numbers
- TLS 1.3's 0-RTT replay protection

## Common Mistakes

1. **Disabling certificate validation** in development and forgetting to enable it
2. **No certificate pinning** on mobile apps (any CA can MITM you)
3. **Ignoring certificate errors** — warnings exist for a reason
4. **Using TLS 1.0/1.1** — both are deprecated and broken
5. **Hardcoded TLS versions** — let the library negotiate the highest version
6. **No hostname verification** — valid cert for wrong domain is still wrong
7. **Replaying 0-RTT data** — TLS 1.3 early data has no replay protection

## Lessons

| # | Lesson | Key Concept |
|---|--------|-------------|
| 01 | TLS Basics | Handshake, key exchange, encrypted channel |
| 02 | Certificate Validation | Chain of trust, expiration, hostname matching |
| 03 | Certificate Pinning | Hardcoded expected cert/public key |
| 04 | Mutual TLS (mTLS) | Both client and server present certificates |
| 05 | Replay Attack | Nonce/timestamp defense against replay |
| 06 | Message Framing | Length prefix, delimiter, injection prevention |
| 07 | DNS Security | DNS spoofing, DNSSEC, DoH/DoT concepts |
| 08 | Secure WebSocket | TLS for persistent connections (wss://) |
| 09 | Channel Binding | Tying auth to TLS channel |
| 10 | Traffic Analysis | Padding, constant-rate, dummy traffic |

## Running

```bash
# Test your implementations
cargo test -p 10-data-in-transit

# Test reference solutions
cargo test -p 10-data-in-transit --features solution

# Run a specific lesson
cargo test -p 10-data-in-transit p01_tls_basics
```
