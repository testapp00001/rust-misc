# Module 29: Capstone — Secure Messenger

> "A secure messaging protocol is only as strong as its weakest link. Build every link yourself before you trust it."

## Overview

This capstone module brings together cryptographic primitives, key management, and protocol design to build a Signal-like secure messenger from scratch. Each lesson builds on the previous, culminating in a full end-to-end encrypted messaging protocol. Topics covered:

- **Key Generation**: Ed25519 identity keys and X25519 ephemeral keys
- **Key Exchange**: X3DH (Extended Triple Diffie-Hellman) protocol
- **Message Encryption**: AES-256-GCM authenticated encryption
- **Message Decryption**: Auth tag verification and plaintext recovery
- **Forward Secrecy**: Double Ratchet key rotation per message
- **Key Verification**: Safety numbers and fingerprint comparison
- **Group Messaging**: Sender keys for efficient group encryption
- **Message Padding**: Traffic analysis resistance via padding
- **Key Backup**: Encrypted key backup with password-derived keys
- **Full Protocol**: Integration of all components into a working protocol

## Key Concepts

### X3DH Key Exchange (Signal Protocol)

```
Alice (Initiator)                    Bob (Responder)
─────────────────                    ───────────────
Identity Key: IKa (Ed25519)          Identity Key: IKb (Ed25519)
Signed Prekey: SPKa (X25519)         Signed Prekey: SPKb (X25519)
One-time Prekey: OPKa (X25519)       One-time Prekey: OPKb (X25519)

DH1 = X25519(IKa, SPKb)
DH2 = X25519(EKa, IKb)
DH3 = X25519(EKa, SPKb)
DH4 = X25519(EKa, OPKb)

SK = HKDF(DH1 || DH2 || DH3 || DH4)
```

### Double Ratchet (Forward Secrecy)

```
Sending Chain:   Ks[0] → Ks[1] → Ks[2] → ...
                      ↓        ↓        ↓
                  msg_key   msg_key   msg_key

Receiving Chain: Kr[0] → Kr[1] → Kr[2] → ...
                      ↓        ↓        ↓
                  msg_key   msg_key   msg_key

DH Ratchet: When direction changes, new DH shared secret
mixes into the chain key, providing forward secrecy across
sending/receiving phases.
```

### Group Messaging with Sender Keys

```
Group Key = random 32-byte secret
Each member receives: Encrypt(GroupKey, to_member_identity_key)

Sending:
  message_key = HKDF(GroupKey || sender_secret || counter)
  ciphertext = AES-256-GCM(message_key, plaintext)

Receiving:
  Derive message_key from GroupKey + sender info
  Decrypt with AES-256-GCM
```

### Safety Numbers (Key Verification)

```
safety_number = Truncate(
    SHA-256(identity_key_A || identity_key_B || sorted(names)),
    60 digits
)

Display as: 12345 67890 12345 67890 12345 67890
            12345 67890 12345 67890 12345 67890

Users compare visually or scan QR code.
```

## Attack Patterns

### Key Compromise Impersonation
If an attacker compromises a long-term identity key, they can impersonate the victim. **Defense**: Use separate identity and ephemeral keys; rotate ephemeral keys frequently.

### Replay Attack
An attacker re-sends a previously captured message. **Defense**: Include monotonic counter or timestamp in each message; reject messages with seen counters.

### Man-in-the-Middle on Key Exchange
Without authentication, an attacker can intercept the key exchange. **Defense**: Identity keys authenticate the exchange; safety numbers let users verify out-of-band.

### Traffic Analysis
Even with encryption, message sizes and timing reveal patterns. **Defense**: Pad messages to fixed sizes; add random delays.

### Group Key Compromise
If a group key leaks, all past messages are exposed. **Defense**: Rotate sender keys per message; use key ratcheting within groups.

## Rust-Specific Tips

1. Use `ed25519-dalek` for identity signing keys — they implement `Signer` and `Verifier` traits
2. Use `x25519-dalek` for Diffie-Hellman key agreement — `EphemeralSecret` ensures keys are zeroized
3. Use `aes-gcm` crate for AEAD encryption — always verify the authentication tag
4. Use `zeroize` on all secret key material — prevents leaks via stack reuse
5. Use `secrecy::Secret` to wrap key bytes — prevents accidental logging or printing
6. Use `hkdf` for key derivation — one master secret becomes multiple purpose-bound keys
7. Prefer `ring` for constant-time operations when comparing tags or hashes

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_key_generation.rs` | Key generation | Ed25519 identity + X25519 ephemeral keypairs |
| 02 | `p02_key_exchange.rs` | Key exchange | X3DH protocol, shared secret derivation |
| 03 | `p03_message_encryption.rs` | Encryption | AES-256-GCM, nonce management, AD |
| 04 | `p04_message_decryption.rs` | Decryption | Auth tag verification, nonce extraction |
| 05 | `p05_forward_secrecy.rs` | Forward secrecy | Double Ratchet, chain key advancement |
| 06 | `p06_key_verification.rs` | Key verification | Safety numbers, fingerprint comparison |
| 07 | `p07_group_messaging.rs` | Group messaging | Sender keys, group key distribution |
| 08 | `p08_message_padding.rs` | Message padding | Traffic analysis resistance |
| 09 | `p09_key_backup.rs` | Key backup | Password-derived encryption, secure backup |
| 10 | `p10_full_protocol.rs` | Full protocol | Complete messenger integration |

## Quick Start

```bash
# Test your implementations
cargo test -p capstone_secure_messenger

# Test reference solutions
cargo test -p capstone_secure_messenger --features solution

# Run a specific lesson's tests
cargo test -p capstone_secure_messenger p01_key_generation
```

## References

- [Signal Protocol Specification](https://signal.org/docs/)
- [X3DH Key Agreement Protocol](https://signal.org/docs/specifications/x3dh/)
- [Double Ratchet Algorithm](https://signal.org/docs/specifications/doubleratchet/)
- [Sender Keys for Group Messaging](https://signal.org/docs/specifications/senderkey/)
- [MLS Protocol (Messaging Layer Security)](https://messaginglayersecurity.rocks/)
- [NIST SP 800-56C: Key Derivation](https://csrc.nist.gov/publications/detail/sp/800-56c/rev-2/final)
