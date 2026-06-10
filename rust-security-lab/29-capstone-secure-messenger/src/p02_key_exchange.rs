//! # Lesson 02: Key Exchange — X3DH Protocol
//!
//! ## What Is X3DH?
//!
//! Extended Triple Diffie-Hellman (X3DH) is the key agreement protocol used by Signal.
//! It establishes a shared secret between two parties, even if one is offline.
//!
//! Unlike basic DH, X3DH uses three (or four) DH computations to resist
//! key-compromise impersonation and provide deniability.
//!
//! ## Protocol Flow
//!
//! ```
//! Alice (Initiator) publishes nothing new.
//! Bob (Responder) has published:
//!   - Identity Key:    IKb  (Ed25519, long-term)
//!   - Signed Prekey:   SPKb (X25519, rotated periodically)
//!   - One-time Prekey: OPKb (X25519, used once, optional)
//!
//! Alice computes:
//!   DH1 = X25519(IKa_secret, SPKb_public)
//!   DH2 = X25519(EKa_secret, IKb_public)    ← ephemeral to identity
//!   DH3 = X25519(EKa_secret, SPKb_public)
//!   DH4 = X25519(EKa_secret, OPKb_public)   ← only if OPK exists
//!
//! SK = HKDF(DH1 || DH2 || DH3 || DH4, info="X3DH")
//! ```
//!
//! ## Attack: Key Compromise Impersonation (KCI)
//!
//! If an attacker compromises Alice's identity key, they can impersonate anyone
//! to Alice. X3DH mitigates this by mixing the ephemeral key into the shared secret —
//! even with the identity key, the attacker cannot compute DH2 or DH3 without the
//! ephemeral secret.
//!
//! ## Attack: Replay
//!
//! Replaying the same X3DH message won't help — each message includes a fresh
//! ephemeral key, and the one-time prekey is consumed.

use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, StaticSecret, SharedSecret};
use rand::rngs::OsRng;
use sha2::Sha256;
use hkdf::Hkdf;

/// A user's prekey bundle, published to a server for X3DH.
///
/// Contains the identity key, a signed prekey (with signature),
/// and optionally a one-time prekey.
#[derive(Clone)]
pub struct PreKeyBundle {
    /// Long-term Ed25519 identity verifying key.
    pub identity_key: Ed25519VerifyingKey,
    /// Signed X25519 prekey (public part).
    pub signed_prekey: X25519PublicKey,
    /// Ed25519 signature over signed_prekey bytes.
    pub signed_prekey_signature: Signature,
    /// Optional one-time X25519 prekey (public part).
    pub one_time_prekey: Option<X25519PublicKey>,
}

/// Represents the result of an X3DH key exchange.
#[derive(Clone)]
pub struct X3DHResult {
    /// The derived shared secret (32 bytes).
    pub shared_secret: [u8; 32],
    /// The initiator's ephemeral public key (sent to responder).
    pub ephemeral_public: X25519PublicKey,
}

/// Exercise 1: Verify the signed prekey in a bundle.
///
/// Check that the signature over `signed_prekey` was produced by `identity_key`.
///
/// Hints:
/// - Convert the X25519 public key bytes and verify the Ed25519 signature
/// - Use `identity_key.verify(signed_prekey_bytes, &signature)`
pub fn verify_signed_prekey(bundle: &PreKeyBundle) -> bool {
    todo!("Verify that the signed prekey signature is valid")
}

/// Exercise 2: Perform the X3DH key exchange (initiator side).
///
/// Alice calls this to compute the shared secret using Bob's prekey bundle.
///
/// Returns the shared secret and the ephemeral public key (to send to Bob).
///
/// Hints:
/// 1. Generate an ephemeral X25519 keypair
/// 2. Compute DH1 = X25519(alice_identity_secret_as_x25519, bundle.signed_prekey)
/// 3. Compute DH2 = X25519(ephemeral_secret, bundle.identity_key_as_x25519)
/// 4. Compute DH3 = X25519(ephemeral_secret, bundle.signed_prekey)
/// 5. Compute DH4 = X25519(ephemeral_secret, bundle.one_time_prekey) if present
/// 6. Concatenate DH1..DH4 and feed into HKDF-SHA256
/// 7. Extract 32-byte shared secret with info="X3DH"
///
/// Note: Converting Ed25519 keys to X25519 requires computing the Montgomery form.
/// For simplicity, you can use `ed25519_dalek::VerifyingKey::to_montgomery()`.
pub fn x3dh_initiate(
    alice_identity: &Ed25519SigningKey,
    bundle: &PreKeyBundle,
) -> X3DHResult {
    todo!("Perform X3DH key exchange as initiator")
}

/// Exercise 3: Perform the X3DH key exchange (responder side).
///
/// Bob calls this to compute the same shared secret using Alice's ephemeral public key.
///
/// Hints:
/// 1. Compute DH1 = X25519(bob_signed_prekey_secret, alice_identity_public_as_x25519)
/// 2. Compute DH2 = X25519(bob_identity_secret_as_x25519, alice_ephemeral_public)
/// 3. Compute DH3 = X25519(bob_signed_prekey_secret, alice_ephemeral_public)
/// 4. Compute DH4 = X25519(bob_one_time_prekey_secret, alice_ephemeral_public) if present
/// 5. Concatenate and HKDF with info="X3DH"
///
/// Note: The DH order matters! DH1 from Bob's perspective is:
///   X25519(bob_signed_prekey_secret, alice_identity_public)
/// which equals X25519(alice_identity_secret, bob_signed_prekey_public) from Alice's side.
pub fn x3dh_respond(
    bob_identity: &Ed25519SigningKey,
    bob_signed_prekey: &StaticSecret,
    bob_one_time_prekey: Option<&StaticSecret>,
    alice_ephemeral_public: &X25519PublicKey,
    alice_identity_public: &Ed25519VerifyingKey,
) -> [u8; 32] {
    todo!("Perform X3DH key exchange as responder")
}

/// Exercise 4: Derive additional keys from the shared secret.
///
/// Use HKDF to derive separate keys for encryption and MAC from the shared secret.
///
/// Returns (encryption_key, mac_key), each 32 bytes.
///
/// Hints:
/// - Use HKDF-SHA256 with the shared secret as IKM
/// - Use different info strings: "encryption" and "mac"
/// - Extract 32 bytes for each key
pub fn derive_session_keys(shared_secret: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    todo!("Derive encryption and MAC keys from shared secret")
}

/// Helper: Convert Ed25519 public key to X25519 public key.
pub fn ed25519_to_x25519_public(key: &Ed25519VerifyingKey) -> X25519PublicKey {
    let montgomery = key.to_montgomery();
    X25519PublicKey::from(montgomery.to_bytes())
}

/// Helper: Convert Ed25519 signing key bytes to X25519 static secret.
pub fn ed25519_to_x25519_secret(key: &Ed25519SigningKey) -> StaticSecret {
    let key_bytes = key.to_bytes();
    let mut scalar = [0u8; 32];
    // Ed25519 to X25519: use the first 32 bytes (the seed) clamped
    scalar.copy_from_slice(&key_bytes[..32]);
    StaticSecret::from(scalar)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identity() -> (Ed25519SigningKey, Ed25519VerifyingKey) {
        let signing = Ed25519SigningKey::generate(&mut OsRng);
        let verifying = signing.verifying_key();
        (signing, verifying)
    }

    fn make_bundle(signing: &Ed25519SigningKey, verifying: &Ed25519VerifyingKey, with_opk: bool) -> PreKeyBundle {
        let spk_secret = StaticSecret::random_from_rng(OsRng);
        let spk_public = X25519PublicKey::from(&spk_secret);
        let spk_sig = signing.sign(spk_public.as_bytes());
        let opk = if with_opk {
            let opk_secret = StaticSecret::random_from_rng(OsRng);
            Some(X25519PublicKey::from(&opk_secret))
        } else {
            None
        };
        PreKeyBundle {
            identity_key: *verifying,
            signed_prekey: spk_public,
            signed_prekey_signature: spk_sig,
            one_time_prekey: opk,
        }
    }

    #[test]
    fn test_verify_signed_prekey_valid() {
        let (signing, verifying) = make_identity();
        let bundle = make_bundle(&signing, &verifying, false);
        assert!(verify_signed_prekey(&bundle), "Valid signature should pass");
    }

    #[test]
    fn test_verify_signed_prekey_tampered() {
        let (signing, verifying) = make_identity();
        let mut bundle = make_bundle(&signing, &verifying, false);
        // Tamper with the signed prekey
        let mut bytes = bundle.signed_prekey.to_bytes();
        bytes[0] ^= 0xFF;
        bundle.signed_prekey = X25519PublicKey::from(bytes);
        assert!(!verify_signed_prekey(&bundle), "Tampered prekey should fail verification");
    }

    #[test]
    fn test_x3dh_shared_secret_matches() {
        let (alice_sign, alice_verify) = make_identity();
        let (bob_sign, bob_verify) = make_identity();

        // Bob publishes his bundle
        let bob_spk_secret = StaticSecret::random_from_rng(OsRng);
        let bob_spk_public = X25519PublicKey::from(&bob_spk_secret);
        let bob_spk_sig = bob_sign.sign(bob_spk_public.as_bytes());

        let bundle = PreKeyBundle {
            identity_key: bob_verify,
            signed_prekey: bob_spk_public,
            signed_prekey_signature: bob_spk_sig,
            one_time_prekey: None,
        };

        // Alice initiates
        let result = x3dh_initiate(&alice_sign, &bundle);

        // Bob responds
        let bob_shared = x3dh_respond(
            &bob_sign,
            &bob_spk_secret,
            None,
            &result.ephemeral_public,
            &alice_verify,
        );

        assert_eq!(
            result.shared_secret, bob_shared,
            "Alice and Bob should derive the same shared secret"
        );
    }

    #[test]
    fn test_x3dh_with_one_time_prekey() {
        let (alice_sign, alice_verify) = make_identity();
        let (bob_sign, bob_verify) = make_identity();

        let bob_spk_secret = StaticSecret::random_from_rng(OsRng);
        let bob_spk_public = X25519PublicKey::from(&bob_spk_secret);
        let bob_spk_sig = bob_sign.sign(bob_spk_public.as_bytes());

        let bob_opk_secret = StaticSecret::random_from_rng(OsRng);
        let bob_opk_public = X25519PublicKey::from(&bob_opk_secret);

        let bundle = PreKeyBundle {
            identity_key: bob_verify,
            signed_prekey: bob_spk_public,
            signed_prekey_signature: bob_spk_sig,
            one_time_prekey: Some(bob_opk_public),
        };

        let result = x3dh_initiate(&alice_sign, &bundle);
        let bob_shared = x3dh_respond(
            &bob_sign,
            &bob_spk_secret,
            Some(&bob_opk_secret),
            &result.ephemeral_public,
            &alice_verify,
        );

        assert_eq!(result.shared_secret, bob_shared);
    }

    #[test]
    fn test_x3dh_different_sessions_different_secrets() {
        let (alice_sign, alice_verify) = make_identity();
        let (bob_sign, bob_verify) = make_identity();

        let bob_spk_secret = StaticSecret::random_from_rng(OsRng);
        let bob_spk_public = X25519PublicKey::from(&bob_spk_secret);
        let bob_spk_sig = bob_sign.sign(bob_spk_public.as_bytes());

        let bundle = PreKeyBundle {
            identity_key: bob_verify,
            signed_prekey: bob_spk_public,
            signed_prekey_signature: bob_spk_sig,
            one_time_prekey: None,
        };

        let result1 = x3dh_initiate(&alice_sign, &bundle);
        let result2 = x3dh_initiate(&alice_sign, &bundle);

        assert_ne!(
            result1.shared_secret, result2.shared_secret,
            "Different ephemeral keys should produce different shared secrets"
        );
    }

    #[test]
    fn test_derive_session_keys_different() {
        let shared = [42u8; 32];
        let (enc, mac) = derive_session_keys(&shared);
        assert_ne!(enc, mac, "Encryption and MAC keys should be different");
        assert_eq!(enc.len(), 32);
        assert_eq!(mac.len(), 32);
    }

    #[test]
    fn test_derive_session_keys_deterministic() {
        let shared = [42u8; 32];
        let (enc1, mac1) = derive_session_keys(&shared);
        let (enc2, mac2) = derive_session_keys(&shared);
        assert_eq!(enc1, enc2);
        assert_eq!(mac1, mac2);
    }
}
