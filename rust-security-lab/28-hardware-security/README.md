# Module 28: Hardware Security

> "Software-only security has limits. Hardware roots of trust anchor the entire security chain from silicon up."

## Overview

Hardware security leverages physical devices and on-chip features to protect secrets, verify integrity, and resist tampering. This module covers:
- **TPM**: Trusted Platform Module — secure key storage and platform integrity measurement
- **HSM**: Hardware Security Module — tamper-resistant key storage for high-assurance cryptography
- **Secure Enclaves**: SGX, TrustZone — isolated execution environments on the CPU
- **Attestation**: Proving to a remote party what software is running
- **Hardware Tokens**: FIDO2, YubiKey — phishing-resistant authentication
- **Secure Boot**: Verifying each boot stage before loading the next
- **TEE**: Trusted Execution Environment — isolated code execution with hardware guarantees
- **Hardware RNG**: True random number generators vs. CSPRNGs
- **Tamper Detection**: Zeroize on physical intrusion

## Key Concepts

### Root of Trust
```
Hardware Root of Trust (TPM / secure element)
  └── Measured Boot → PCR values
       └── Attestation → remote verification
            └── Key Release → secrets unlocked only on known-good state
```

### TPM vs HSM vs Secure Enclave

| Property | TPM | HSM | Secure Enclave |
|----------|-----|-----|----------------|
| Primary purpose | Platform integrity | Key management | Isolated execution |
| Form factor | On motherboard / firmware | PCIe card / network appliance | On CPU die |
| Key storage | Limited (24-150 keys) | Millions of keys | Limited |
| Tamper resistance | Moderate | High (FIPS 140-2/3) | High (hardware isolation) |
| Performance | Slow | Fast (dedicated crypto) | Moderate |
| Typical use | Measured boot, disk encryption | PKI, payment processing | Secret computation |

### Attestation Flow
```
1. Platform boots → TPM extends PCR registers with measurements
2. Application requests attestation quote (PCR values + signature)
3. Quote sent to remote verifier
4. Verifier checks: signature valid? PCR values expected?
5. If good → release secrets / grant access
```

### Secure Boot Chain
```
ROM (immutable) → verifies → Bootloader
Bootloader → verifies → OS kernel
OS kernel → verifies → Drivers / modules
Each stage: hash, verify signature, load only if valid
```

### Hardware vs Software RNG

| Property | TRNG (Hardware) | CSPRNG (Software) |
|----------|----------------|-------------------|
| Entropy source | Physical phenomenon (thermal noise, jitter) | Seed from TRNG |
| Speed | Slower (hardware-dependent) | Fast |
| Deterministic | No | Yes (seed-dependent) |
| Use case | Seeding CSPRNG, key generation | Bulk random data |
| Example | Intel RDRAND, /dev/hwrng | ChaCha20-based PRNG |

## Attack Patterns

### TPM Reset Attack
An attacker with physical access may try to reset the TPM to clear PCR measurements, allowing a malicious boot chain to appear legitimate. **Defense**: Use TPM2 with policy sessions that require specific PCR values.

### Side-Channel on Enclave
Even SGX enclaves can leak secrets through cache timing, power analysis, or speculative execution (Spectre/Meltdown). **Defense**: Constant-time code, cache line flushing, microcode updates.

### Token Cloning
Hardware tokens can be cloned if the secret key is extracted (e.g., via fault injection). **Defense**: Secure elements with active tamper mesh, keys marked as non-exportable.

### Bootkit / Secure Boot Bypass
An attacker modifies the bootloader to load a rootkit before the OS. **Defense**: UEFI Secure Boot with custom keys, Measured Boot with TPM attestation.

## Rust-Specific Tips

1. Use `ring` for crypto operations — it abstracts over hardware acceleration (AES-NI, etc.)
2. Use `zeroize` crate for secure memory cleanup — prevents compiler from optimizing away wipes
3. Use `rand::rngs::OsRng` for entropy from the OS (which may use hardware RNG)
4. Model hardware as traits — allows testing with mock implementations
5. Use `serde` to serialize attestation documents and platform configs

## Lesson Table

| # | File | Topic | Key Concept |
|---|------|-------|-------------|
| 01 | `p01_tpm_basics.rs` | TPM concepts | Secure key storage, PCR registers, platform integrity |
| 02 | `p02_hsm_concepts.rs` | HSM concepts | Tamper-resistant key storage, crypto offload |
| 03 | `p03_secure_enclave.rs` | Secure enclaves | SGX, TrustZone, isolated execution |
| 04 | `p04_attestation.rs` | Remote attestation | Prove software integrity to remote verifier |
| 05 | `p05_hardware_tokens.rs` | Hardware tokens | FIDO2, YubiKey, phishing-resistant auth |
| 06 | `p06_secure_boot.rs` | Secure boot | Verify each boot stage before loading next |
| 07 | `p07_trusted_execution.rs` | TEE | Trusted Execution Environment |
| 08 | `p08_hardware_rng.rs` | Hardware RNG | TRNG vs CSPRNG, entropy quality |
| 09 | `p09_tamper_detection.rs` | Tamper detection | Zeroize on physical tamper |
| 10 | `p10_hardware_integration.rs` | Platform abstraction | Integrating hardware security in Rust |

## References

- [TCG TPM 2.0 Specification](https://trustedcomputinggroup.org/resource/tpm-library-specification/)
- [NIST FIPS 140-3](https://csrc.nist.gov/publications/detail/fips/140/3/final)
- [Intel SGX Explained](https://eprint.iacr.org/2016/086.pdf)
- [FIDO2/WebAuthn Specification](https://fidoalliance.org/fido2/)
- [ARM TrustZone](https://developer.arm.com/Architectures/TrustZone)
- [UEFI Secure Boot](https://learn.microsoft.com/en-us/windows-hardware/design/secure/boot-secure-boot)
