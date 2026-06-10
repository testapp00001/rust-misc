# Module 27: Advanced Cryptographic Protocols

> "The good news about cryptography is that we already have the algorithms and protocols we need to secure our systems. The bad news is that that was true in 1995 as well." -- Bruce Schneier

## Overview

Advanced cryptographic protocols build on primitives (encryption, hashing, signatures) to solve complex multi-party security problems. This module covers:

- **Secret Sharing**: Shamir's scheme -- split a secret into N shares, any K can reconstruct
- **Multi-Party Computation**: Compute a function over private inputs without revealing them
- **Homomorphic Encryption**: Compute on ciphertexts, decrypt to get the result of computation on plaintexts
- **Oblivious Transfer**: Send one of N messages, receiver learns only their choice
- **Secure Aggregation**: Sum private values without revealing individual contributions
- **Mental Poker**: Play card games without a trusted dealer
- **Commitment Schemes**: Pedersen and ElGamal -- commit to a value without revealing it
- **Zero-Knowledge for Arbitrary Statements**: Prove any NP statement without revealing the witness
- **Blind Signatures**: Sign a message without seeing its content
- **Anonymous Credentials**: Prove attributes (e.g., age > 18) without revealing identity

## Key Concepts

### Shamir's Secret Sharing (SSS)

Based on polynomial interpolation over a finite field. A secret `s` is hidden as the constant term of a random polynomial of degree k-1:

```
f(x) = s + a1*x + a2*x^2 + ... + a(k-1)*x^(k-1)  (mod p)
```

Each share is a point `(i, f(i))`. Any k points reconstruct the polynomial via Lagrange interpolation; fewer than k points reveal nothing about s.

```
n = total shares, k = threshold
k=1  -->  no security (any single share reveals the secret)
k=n  -->  all shares required (one lost = secret lost)
k=3, n=5  -->  any 3 of 5 shares suffice
```

### Secure Multi-Party Computation (MPC)

N parties each hold private input x_i. They want to compute f(x_1, ..., x_n) without revealing individual x_i. Core building blocks:

1. **Garbled Circuits**: One party encrypts a Boolean circuit; the other evaluates it obliviously
2. **Secret Sharing**: Distribute shares, compute on shares, reconstruct result
3. **Homomorphic Encryption**: Encrypt inputs, compute on ciphertexts

### Homomorphic Encryption (HE)

| Type | Operations | Example |
|------|-----------|---------|
| Partially HE (PHE) | One operation (add OR multiply) | Paillier (add), RSA (multiply) |
| Somewhat HE (SHE) | Limited depth of both | BGV, BFV |
| Fully HE (FHE) | Unlimited add + multiply | CKKS, TFHE |

The dream: outsource computation to the cloud without ever decrypting your data.

### Oblivious Transfer (OT)

Sender has messages m_0, m_1. Receiver has choice bit b. After OT:
- Receiver learns m_b but nothing about m_(1-b)
- Sender learns nothing about b

1-out-of-2 OT is the fundamental building block for garbled circuits and MPC.

### Mental Poker

Play poker over a network without a trusted dealer. Requirements:
1. Cards are encrypted/shuffled so no one knows the order
2. Players can verify the deck is fair (52 distinct cards)
3. Players can privately look at their own hand
4. No player can cheat by claiming a different card

### Commitment Schemes

A commitment to value v has two phases:
1. **Commit**: Send C = Commit(v, r) where r is randomness
2. **Reveal**: Open by revealing v and r; verifier checks C = Commit(v, r)

Properties:
- **Hiding**: C reveals nothing about v before opening
- **Binding**: Cannot open C to a different value v'

Pedersen commitment: C = g^v * h^r (mod p), information-theoretically hiding, computationally binding.

### Blind Signatures

A voter wants a signature on ballot B, but the signer must not learn which ballot was signed. The voter "blinds" the message, the signer signs the blinded version, and the voter "unblinds" to get a valid signature on the original.

Used in: e-voting, anonymous digital cash, privacy-preserving authentication.

### Anonymous Credentials

Prove statements about credentials without revealing the credential itself:
- "I am over 18" without revealing exact age or identity
- "I am a member of group X" without revealing which member
- Based on zero-knowledge proofs of knowledge (e.g., Camenisch-Lysyanskaya signatures)

## Attack Patterns

### Share Collusion
In (k, n) secret sharing, k-1 colluding parties learn nothing. But k parties can reconstruct the secret. The threshold must be chosen carefully -- too low and collusion breaks security.

### MPC Malicious vs Semi-Honest
- **Semi-honest (honest-but-curious)**: Parties follow the protocol but try to learn extra info from messages they see
- **Malicious**: Parties may deviate from the protocol arbitrarily
- Malicious MPC requires additional mechanisms (zero-knowledge proofs, MACs on shares)

### Commitment Binding Attacks
If the commitment scheme is not binding, the committer can open to a different value. This breaks auction protocols (change bid after seeing others), coin flipping (change outcome), and voting.

### Blinding Factor Reuse
In blind signatures, reusing the blinding factor links different signed messages, destroying anonymity. Each blinding must use fresh randomness.

## Rust-Specific Tips

1. Use `rand::thread_rng()` for all randomness generation (never hardcode nonces)
2. Use `num-bigint` or manual modular arithmetic for finite field operations (the exercises use i64 for simplicity)
3. Use `sha2::Sha256` for hashing to derive deterministic values
4. Use `ring` for HMAC-based operations when applicable
5. Use `serde` for serializing protocol messages
6. Modular arithmetic with negative numbers: `((a % m) + m) % m` to ensure positive remainder

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_secret_sharing.rs` | Shamir's Secret Sharing | Split into N shares, K to reconstruct |
| 02 | `p02_multi_party_compute.rs` | Secure Multi-Party Computation | Compute over private inputs |
| 03 | `p03_homomorphic_concepts.rs` | Homomorphic Encryption | Compute on ciphertexts |
| 04 | `p04_oblivious_transfer.rs` | Oblivious Transfer | Send without learning choice |
| 05 | `p05_secure_aggregation.rs` | Secure Aggregation | Sum private values |
| 06 | `p06_mental_poker.rs` | Mental Poker | Cards without trusted dealer |
| 07 | `p07_commitment_schemes_advanced.rs` | Commitment Schemes | Pedersen, ElGamal commitments |
| 08 | `p08_zero_knowledge_advanced.rs` | ZK for Arbitrary Statements | Prove NP statements |
| 09 | `p09_blind_signatures.rs` | Blind Signatures | Sign without seeing |
| 10 | `p10_anonymous_credentials.rs` | Anonymous Credentials | Prove attributes anonymously |

## Usage

```bash
# Test your implementation (exercise stubs)
cargo test -p advanced_crypto_protocols

# Test the reference solution
cargo test -p advanced_crypto_protocols --features solution
```

## References

- [Shamir, "How to Share a Secret" (1979)](https://dl.acm.org/doi/10.1145/359168.359176)
- [Goldreich, "Foundations of Cryptography, Volume 2"](https://www.wisdom.weizmann.ac.il/~oded/foc-vol2.html)
- [Gentry, "Fully Homomorphic Encryption Using Ideal Lattices"](https://dl.acm.org/doi/10.1145/1806689.1806713)
- [Chaum, "Blind Signatures for Untraceable Payments"](https://sceweb.sce.uhcl.edu/yang/teaching/csci5234WebSecurityFall2011/Chaum-blind-signatures.PDF)
- [Camenisch & Lysyanskaya, "An Efficient System for Non-transferable Anonymous Credentials"](https://link.springer.com/chapter/10.1007/3-540-44987-6_7)
- [Pedersen, "Non-Interactive and Information-Theoretic Secure Verifiable Secret Sharing"](https://link.springer.com/chapter/10.1007/3-540-46766-1_9)
- [Rabin, "How to Exchange Secrets with Oblivious Transfer"](https://eprint.iacr.org/2005/187.pdf)
