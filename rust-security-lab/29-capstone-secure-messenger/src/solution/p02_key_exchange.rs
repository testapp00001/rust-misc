//! # Lesson 02: Key Exchange — X3DH Protocol (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ed25519_dalek::{SigningKey as Ed25519SigningKey, VerifyingKey as Ed25519VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use rand::rngs::OsRng;
use hkdf::Hkdf;
use sha2::Sha256;

/// A user's prekey bundle, published to a server for X3DH.
#[derive(Clone)]
pub struct PreKeyBundle {
    pub identity_key: Ed25519VerifyingKey,
    pub signed_prekey: X25519PublicKey,
    pub signed_prekey_signature: Signature,
    pub one_time_prekey: Option<X25519PublicKey>,
}

/// The result of an X3DH key exchange.
#[derive(Clone)]
pub struct X3DHResult {
    pub shared_secret: [u8; 32],
    pub ephemeral_public: X25519PublicKey,
}

/// Verify the signed prekey in a bundle.
pub fn verify_signed_prekey(bundle: &PreKeyBundle) -> bool {
    bundle
        .identity_key
        .verify(bundle.signed_prekey.as_bytes(), &bundle.signed_prekey_signature)
        .is_ok()
}

/// Perform the X3DH key exchange (initiator side).
pub fn x3dh_initiate(
    alice_identity: &Ed25519SigningKey,
    bundle: &PreKeyBundle,
) -> X3DHResult {
    // Generate ephemeral keypair (use StaticSecret for multiple DH calls)
    let ek_secret = StaticSecret::random_from_rng(OsRng);
    let ek_public = X25519PublicKey::from(&ek_secret);

    // Convert Alice's identity to X25519
    let alice_id_secret = ed25519_to_x25519_secret(alice_identity);
    let bob_id_public = ed25519_to_x25519_public(&bundle.identity_key);

    // DH1 = X25519(alice_identity_secret, bob_signed_prekey)
    let dh1 = alice_id_secret.diffie_hellman(&bundle.signed_prekey);

    // DH2 = X25519(ephemeral_secret, bob_identity_public)
    let dh2 = ek_secret.diffie_hellman(&bob_id_public);

    // DH3 = X25519(ephemeral_secret, bob_signed_prekey)
    let dh3 = ek_secret.diffie_hellman(&bundle.signed_prekey);

    // Concatenate DH outputs
    let mut ikm = Vec::new();
    ikm.extend_from_slice(dh1.as_bytes());
    ikm.extend_from_slice(dh2.as_bytes());
    ikm.extend_from_slice(dh3.as_bytes());

    // DH4 if one-time prekey exists
    if let Some(opk) = &bundle.one_time_prekey {
        let dh4 = ek_secret.diffie_hellman(opk);
        ikm.extend_from_slice(dh4.as_bytes());
    }

    // HKDF to derive shared secret
    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut shared_secret = [0u8; 32];
    hk.expand(b"X3DH", &mut shared_secret).expect("HKDF expand failed");

    X3DHResult {
        shared_secret,
        ephemeral_public: ek_public,
    }
}

/// Perform the X3DH key exchange (responder side).
pub fn x3dh_respond(
    bob_identity: &Ed25519SigningKey,
    bob_signed_prekey: &StaticSecret,
    bob_one_time_prekey: Option<&StaticSecret>,
    alice_ephemeral_public: &X25519PublicKey,
    alice_identity_public: &Ed25519VerifyingKey,
) -> [u8; 32] {
    let alice_id_public = ed25519_to_x25519_public(alice_identity_public);
    let bob_id_secret = ed25519_to_x25519_secret(bob_identity);

    // DH1 = X25519(bob_signed_prekey, alice_identity_public)
    let dh1 = bob_signed_prekey.diffie_hellman(&alice_id_public);

    // DH2 = X25519(bob_identity_secret, alice_ephemeral_public)
    let dh2 = bob_id_secret.diffie_hellman(alice_ephemeral_public);

    // DH3 = X25519(bob_signed_prekey, alice_ephemeral_public)
    let dh3 = bob_signed_prekey.diffie_hellman(alice_ephemeral_public);

    let mut ikm = Vec::new();
    ikm.extend_from_slice(dh1.as_bytes());
    ikm.extend_from_slice(dh2.as_bytes());
    ikm.extend_from_slice(dh3.as_bytes());

    // DH4 if one-time prekey exists
    if let Some(opk) = bob_one_time_prekey {
        let dh4 = opk.diffie_hellman(alice_ephemeral_public);
        ikm.extend_from_slice(dh4.as_bytes());
    }

    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let mut shared_secret = [0u8; 32];
    hk.expand(b"X3DH", &mut shared_secret).expect("HKDF expand failed");

    shared_secret
}

/// Derive additional keys from the shared secret.
pub fn derive_session_keys(shared_secret: &[u8; 32]) -> ([u8; 32], [u8; 32]) {
    let hk = Hkdf::<Sha256>::new(None, shared_secret);
    let mut enc_key = [0u8; 32];
    let mut mac_key = [0u8; 32];
    hk.expand(b"encryption", &mut enc_key).expect("HKDF expand failed");
    hk.expand(b"mac", &mut mac_key).expect("HKDF expand failed");
    (enc_key, mac_key)
}

/// Helper: Convert Ed25519 public key to X25519 public key.
pub fn ed25519_to_x25519_public(key: &Ed25519VerifyingKey) -> X25519PublicKey {
    let montgomery = key.to_montgomery();
    X25519PublicKey::from(montgomery.to_bytes())
}

/// Helper: Convert Ed25519 signing key to X25519 static secret.
///
/// Uses the Ed25519 scalar bytes (from SHA-512(seed)[0..32]) as the X25519 secret.
/// This is the standard Ed25519-to-X25519 key conversion.
pub fn ed25519_to_x25519_secret(key: &Ed25519SigningKey) -> StaticSecret {
    let scalar_bytes = key.to_scalar_bytes();
    StaticSecret::from(scalar_bytes)
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
        let mut bytes = bundle.signed_prekey.to_bytes();
        bytes[0] ^= 0xFF;
        bundle.signed_prekey = X25519PublicKey::from(bytes);
        assert!(!verify_signed_prekey(&bundle), "Tampered prekey should fail verification");
    }

    #[test]
    fn test_x3dh_shared_secret_matches() {
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

        let result = x3dh_initiate(&alice_sign, &bundle);
        let bob_shared = x3dh_respond(
            &bob_sign,
            &bob_spk_secret,
            None,
            &result.ephemeral_public,
            &alice_verify,
        );

        assert_eq!(result.shared_secret, bob_shared);
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

        assert_ne!(result1.shared_secret, result2.shared_secret);
    }

    #[test]
    fn test_derive_session_keys_different() {
        let shared = [42u8; 32];
        let (enc, mac) = derive_session_keys(&shared);
        assert_ne!(enc, mac);
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
