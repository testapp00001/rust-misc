//! # Lesson 04: Length Extension Attack
//!
//! ## The Attack
//!
//! SHA-256 uses the Merkle-Damgard construction. The internal state after processing
//! a message IS the hash output. This means if you know `SHA256(key || msg)`, you can
//! compute `SHA256(key || msg || padding || extension)` WITHOUT knowing the key.
//!
//! This is called a **length extension attack**.
//!
//! ## How It Works
//!
//! ```text
//! 1. Attacker knows: h = SHA256(key || msg), and len(key || msg)
//! 2. SHA256's internal state after processing (key || msg) is exactly h
//! 3. Attacker can "resume" the hash from state h, feeding it (padding || extension)
//! 4. Result: SHA256(key || msg || padding || extension) — valid, without knowing key!
//! ```
//!
//! ## Why This Matters
//!
//! Many naive authentication schemes use `MAC = H(key || message)`. An attacker who
//! intercepts a valid MAC can forge MACs for extended messages.
//!
//! Real-world example:
//! ```text
//! Original: GET /api/transfer?to=alice&amount=100  →  HMAC = SHA256(secret || query)
//! Forged:   GET /api/transfer?to=alice&amount=100&admin=true  →  valid HMAC!
//! ```
//!
//! ## The Defense: HMAC
//!
//! HMAC uses a double-hashing structure that prevents length extension:
//! ```text
//! HMAC(key, msg) = H((key ^ opad) || H((key ^ ipad) || msg))
//! ```
//!
//! Even if an attacker can extend the inner hash, they cannot compute the outer hash
//! without knowing the key.
//!
//! ## Alternative Defense: Use SHA-3
//!
//! SHA-3 (sponge construction) is inherently immune to length extension attacks.
//! The final output is not the raw internal state.
//!
//! ## Key Takeaway
//!
//! NEVER use `H(key || message)` for authentication. Use HMAC or a dedicated MAC.

use ring::digest;

/// Exercise 1: Compute SHA-256 of (key || message) — the INSECURE way.
///
/// This is the vulnerable construction: just hashing key prepended to message.
///
/// Hints:
/// - Create a new `digest::Context` with SHA256
/// - Update with key, then update with message
/// - Finalize and return the bytes
pub fn insecure_hash_mac(key: &[u8], message: &[u8]) -> Vec<u8> {
    todo!("Compute SHA256(key || message) — the insecure way")
}

/// Exercise 2: Simulate a length extension attack.
///
/// Given the hash of (key || msg), the known msg length, and a secret key length,
/// compute the hash of (key || msg || padding || extension) without knowing the key.
///
/// This works because SHA-256's Merkle-Damgard construction lets us resume
/// hashing from any intermediate state.
///
/// Hints:
/// - Use `ring::digest::Context` with `with_initial_state()` (not available in ring)
/// - Instead, we'll reconstruct: build the full padded message and hash it
/// - The SHA-256 padding is: 0x80, then zeros, then 8-byte big-endian bit length
/// - Total padded length must be a multiple of 64 bytes
///
/// Parameters:
/// - `known_hash`: the SHA-256 output (32 bytes) = SHA256(key || msg)
/// - `key_len`: length of the secret key (attacker knows this)
/// - `msg`: the original message bytes (known to attacker)
/// - `extension`: the evil data to append
///
/// Returns: (forged_hash, forged_message_with_padding_and_extension)
///
/// The forged_hash should equal SHA256(key || msg || padding || extension)
/// for ANY key of the given length.
pub fn length_extension_attack(
    known_hash: &[u8],
    key_len: usize,
    msg: &[u8],
    extension: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    // Build the original message with key prefix (attacker doesn't know key,
    // but knows its length, so uses dummy bytes for padding calculation)
    //
    // Step 1: Build the padded (key || msg) block
    //   total_len = key_len + msg.len()
    //   SHA-256 padding: append 0x80, then zeros until (total % 64 == 56),
    //   then append total_len * 8 as 8-byte big-endian
    //
    // Step 2: Append the extension
    //
    // Step 3: Hash the whole thing with a fake key of the right length
    //
    // Hints:
    // - Compute the padding for (key_len + msg.len()) bytes
    // - Build: dummy_key || msg || sha256_padding || extension
    // - Hash this whole thing — the result will match SHA256(key || msg || padding || extension)
    //   for any key of the same length

    todo!("Implement the length extension attack")
}

/// Exercise 3: Demonstrate that HMAC prevents length extension.
///
/// Given an HMAC tag, show that we cannot extend the message and produce a valid HMAC.
///
/// Hints:
/// - This function just computes HMAC(key, msg || extension) using the real key
/// - The point is that an attacker CANNOT do this without the key
/// - The function returns the HMAC of the extended message (only possible with the key)
pub fn hmac_prevents_extension(key: &[u8], msg: &[u8], extension: &[u8]) -> Vec<u8> {
    todo!("Show HMAC prevents length extension (requires key)")
}

/// Exercise 4: Compute the SHA-256 padding for a given message length.
///
/// SHA-256 padding rules:
/// 1. Append bit '1' (byte 0x80)
/// 2. Append zeros until message length ≡ 56 (mod 64)
/// 3. Append original message length in bits as 8-byte big-endian
///
/// Returns the padding bytes (not including the message itself).
pub fn sha256_padding(total_len: usize) -> Vec<u8> {
    todo!("Compute SHA-256 padding for a given total message length")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::hmac;

    #[test]
    fn test_insecure_hash_mac_works() {
        let key = b"secret_key";
        let message = b"hello";
        let hash = insecure_hash_mac(key, message);
        assert_eq!(hash.len(), 32);

        // Verify it matches manual computation
        let mut ctx = digest::Context::new(&digest::SHA256);
        ctx.update(key);
        ctx.update(message);
        let expected = ctx.finish().as_ref().to_vec();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_padding_correct() {
        // "hello" is 5 bytes. After padding:
        // 0x80 + 50 zeros + 8-byte length = 56 bytes total (mod 64 = 56)
        // So we need 56 - 5 = 51 padding bytes, then 8 more for length = 59 padding bytes
        let padding = sha256_padding(5);
        let padded = [b"hello".as_ref(), &padding].concat();
        assert_eq!(padded.len() % 64, 0, "Padded message must be multiple of 64");
        assert_eq!(padding[0], 0x80, "First padding byte must be 0x80");
    }

    #[test]
    fn test_length_extension_attack_works() {
        // Setup: server uses SHA256(key || msg) as MAC
        let key = b"super_secret_key_1234567890"; // 26 bytes
        let msg = b"from=alice&to=bob&amount=100";
        let original_hash = insecure_hash_mac(key, msg);

        // Attacker knows: original_hash, msg, and key length (e.g., from timing or leaked info)
        let (forged_hash, forged_payload) = length_extension_attack(
            &original_hash,
            key.len(),
            msg,
            b"&admin=true",
        );

        // The forged hash should match what the server would compute
        // for key || msg || padding || "&admin=true"
        let full_message = [key.as_ref(), msg, &sha256_padding(key.len() + msg.len()), b"&admin=true"].concat();
        let expected_hash = {
            let mut ctx = digest::Context::new(&digest::SHA256);
            ctx.update(&full_message);
            ctx.finish().as_ref().to_vec()
        };

        // Note: forged_hash should be the SHA256 of the extended message
        // The forged_payload is msg || padding || extension (what attacker sends)
        // Server would hash key || forged_payload
        let server_hash = insecure_hash_mac(key, &forged_payload);

        // The attack succeeds if forged_hash == server_hash
        assert_eq!(forged_hash, server_hash, "Length extension attack should produce valid MAC");
        assert_eq!(forged_hash, expected_hash);
    }

    #[test]
    fn test_hmac_prevents_extension() {
        let key = b"secret_key_for_hmac";
        let msg = b"data";
        let extension = b"evil_extension";

        // Compute HMAC of extended message (requires the key)
        let hmac_extended = hmac_prevents_extension(key, msg, extension);

        // Verify it matches direct HMAC computation
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let expected = hmac::sign(&hmac_key, &[msg, extension].concat());
        assert_eq!(hmac_extended, expected.as_ref().to_vec());
    }

    #[test]
    fn test_length_extension_does_not_apply_to_hmac() {
        let key = b"my_secret_key_1234567890123456"; // 30 bytes
        let msg = b"original message";
        let extension = b"appended evil";

        // HMAC of original message
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let original_hmac = hmac::sign(&hmac_key, msg);

        // HMAC of extended message
        let extended_msg = [msg.as_ref(), extension].concat();
        let extended_hmac = hmac::sign(&hmac_key, &extended_msg);

        // These should be completely different — you cannot derive one from the other
        assert_ne!(
            original_hmac.as_ref(),
            extended_hmac.as_ref(),
            "HMAC tags for different messages must differ"
        );
    }

    #[test]
    fn test_padding_multiple_of_64() {
        // Padding should always make total length a multiple of 64
        for total_len in [0, 1, 55, 56, 57, 63, 64, 65, 127, 128, 1000] {
            let padding = sha256_padding(total_len);
            assert_eq!(
                (total_len + padding.len()) % 64,
                0,
                "Failed for total_len={}",
                total_len
            );
        }
    }

    #[test]
    fn test_padding_contains_length_bits() {
        // The last 8 bytes of padding should encode the bit length
        let total_len: usize = 10;
        let padding = sha256_padding(total_len);
        let bit_len = (total_len as u64) * 8;
        let last_8 = &padding[padding.len() - 8..];
        let encoded = u64::from_be_bytes(last_8.try_into().unwrap());
        assert_eq!(encoded, bit_len, "Last 8 bytes should encode bit length");
    }

    #[test]
    fn test_attack_with_different_key_lengths() {
        // The attack should work regardless of key length
        for key_len in [8, 16, 32, 64] {
            let key = vec![0x42u8; key_len];
            let msg = b"test message";
            let hash = insecure_hash_mac(&key, msg);

            let (forged_hash, forged_payload) = length_extension_attack(
                &hash,
                key_len,
                msg,
                b"evil",
            );

            let server_hash = insecure_hash_mac(&key, &forged_payload);
            assert_eq!(
                forged_hash, server_hash,
                "Attack should work with key_len={}",
                key_len
            );
        }
    }
}
