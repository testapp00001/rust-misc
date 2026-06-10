# Module 05: Zero-Knowledge Proofs

## What is a Zero-Knowledge Proof?

A **zero-knowledge proof** (ZKP) is a cryptographic protocol where one party (the **prover**)
convinces another party (the **verifier**) that a statement is true, *without revealing any
information beyond the truth of the statement itself*.

### The Classic Analogy

Imagine a colorblind friend holding two balls — one red, one green. You want to prove the balls
are different colors without telling your friend which is which. You let your friend hide the balls
behind their back, optionally swap them, then show you the balls again. If you can always say
whether they swapped or not, you prove you can distinguish the colors — but your friend learns
nothing about *which* ball is *which* color.

### The Three Properties

Every zero-knowledge proof must satisfy three properties:

| Property | Meaning | Failure Mode |
|----------|---------|--------------|
| **Completeness** | An honest prover with a valid witness always convinces an honest verifier | Legitimate provers get rejected |
| **Soundness** | A dishonest prover without a valid witness cannot convince the verifier (except with negligible probability) | Cheaters can fool the verifier |
| **Zero-Knowledge** | The verifier learns nothing beyond the truth of the statement | The protocol leaks information about the secret |

## Interactive vs Non-Interactive Proofs

- **Interactive proofs**: Multiple rounds of communication between prover and verifier (e.g., Schnorr identification)
- **Non-interactive proofs**: A single message from prover to verifier, made possible by the **Fiat-Shamir heuristic** (replacing the verifier's random challenges with hash outputs)

## Applications

| Domain | Use Case |
|--------|----------|
| **Authentication** | Prove you know a password without sending it |
| **Privacy-preserving transactions** | Prove a transaction is valid without revealing amounts (Zcash, Monero) |
| **Blockchain scaling** | Prove a batch of transactions is valid with a tiny proof (zk-Rollups) |
| **Identity** | Prove you're over 18 without revealing your birthdate |
| **Voting** | Prove your vote was counted without revealing who you voted for |
| **Supply chain** | Prove a product meets criteria without revealing supplier details |

## Real-World ZK Systems

| System | Type | Trusted Setup | Proof Size | Verification Time |
|--------|------|---------------|------------|-------------------|
| **Groth16 (zk-SNARK)** | SNARK | Yes (per-circuit) | ~200 bytes | ~ms |
| **PLONK** | SNARK | Yes (universal) | ~400 bytes | ~ms |
| **Bulletproofs** | Proof | No | ~1-2 KB | ~seconds |
| **zk-STARKs** | STARK | No | ~50-200 KB | ~ms |

- **zk-SNARKs**: Succinct Non-interactive ARguments of Knowledge — small proofs, fast verification, but require trusted setup
- **zk-STARKs**: Scalable Transparent ARguments of Knowledge — no trusted setup, post-quantum secure, larger proofs
- **Bulletproofs**: Efficient range proofs without trusted setup, used in Monero

## When to Use ZK (and When Not To)

**Use ZK when:**
- You need to prove a property without revealing the underlying data
- Privacy is critical (financial, medical, identity data)
- You need to compress verification work (zk-Rollups)

**Use simpler crypto when:**
- A digital signature suffices (prove authorship)
- A MAC or hash commitment suffices (prove integrity)
- Encryption hides the data (you don't need to prove properties about it)
- A trusted third party can mediate

## Module Contents

| Lesson | Topic | Key Concept |
|--------|-------|-------------|
| p01 | ZK Fundamentals | Completeness, soundness, zero-knowledge; hash preimage proof |
| p02 | Schnorr Proof | Prove knowledge of discrete log without revealing it |
| p03 | Commitment Schemes | Pedersen commitment — binding and hiding |
| p04 | Sigma Protocols | Commit-challenge-response; Fiat-Shamir transform |
| p05 | Range Proofs | Prove a value is in a range without revealing it |
| p06 | Proof of Knowledge | Proving knowledge of a secret vs. proving a statement |
| p07 | Non-Interactive Proofs | Fiat-Shamir heuristic in detail |
| p08 | ZK Set Membership | Merkle tree + ZK proof of membership |
| p09 | ZK Boolean Circuits | Circuit-based ZK for arbitrary computation |
| p10 | Bulletproofs Intro | Efficient range proofs without trusted setup |

## Quick Start

```bash
# Test your implementations
cargo test -p 05-zero-knowledge-proofs

# Test reference solutions
cargo test -p 05-zero-knowledge-proofs --features solution
```

## Prerequisites

- Module 01: Cryptographic Primitives (hashing)
- Module 04: Digital Signatures (elliptic curve basics, Schnorr signatures)
- Basic modular arithmetic (addition, multiplication, modular exponentiation)
