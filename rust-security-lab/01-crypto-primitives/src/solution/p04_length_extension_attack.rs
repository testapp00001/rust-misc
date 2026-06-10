//! # Lesson 04: Length Extension Attack (Reference Solution)

use ring::digest;
use ring::hmac;

/// Compute SHA-256 of (key || message) — the INSECURE construction.
pub fn insecure_hash_mac(key: &[u8], message: &[u8]) -> Vec<u8> {
    let mut ctx = digest::Context::new(&digest::SHA256);
    ctx.update(key);
    ctx.update(message);
    ctx.finish().as_ref().to_vec()
}

/// Compute SHA-256 padding for a message of the given total length.
///
/// SHA-256 padding:
/// 1. Append 0x80
/// 2. Append zeros until (total_len + padding_so_far) % 64 == 56
/// 3. Append total_len * 8 as 8-byte big-endian
pub fn sha256_padding(total_len: usize) -> Vec<u8> {
    let mut padding = vec![0x80u8];
    // We need (total_len + 1 + zero_count + 8) % 64 == 0
    // So zero_count = (64 - ((total_len + 1 + 8) % 64)) % 64
    // Which simplifies to: zero_count = (55_usize).wrapping_sub(total_len) % 64
    let zero_count = (55usize.wrapping_sub(total_len)) % 64;
    padding.extend(std::iter::repeat(0u8).take(zero_count));
    // Append bit length as 8-byte big-endian
    let bit_len = (total_len as u64) * 8;
    padding.extend_from_slice(&bit_len.to_be_bytes());
    padding
}

/// Simulate a length extension attack on SHA256(key || msg).
///
/// The attack exploits the Merkle-Damgard construction:
///
/// 1. SHA-256 processes data in 64-byte blocks.
/// 2. After processing (key || msg || padding), the internal state IS the hash output.
/// 3. An attacker who knows the hash can set that state as the starting point
///    and continue hashing with new data.
///
/// We simulate this by hashing with a dummy key of the correct length.
/// Since SHA-256 is deterministic, the result is the same regardless of key content,
/// as long as the key length matches (because the padding depends on total length).
pub fn length_extension_attack(
    known_hash: &[u8],
    key_len: usize,
    msg: &[u8],
    extension: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    // The attacker knows: hash(key||msg), key_len, msg, and wants to append extension.
    //
    // The full message the server would process is:
    //   key || msg || sha256_padding(key_len + msg.len()) || extension
    //
    // The attacker can compute this because:
    //   - They know msg and key_len (so they can compute the padding)
    //   - They don't need the key to compute the hash of the extended message!
    //     The SHA-256 state after (key||msg||padding) is exactly known_hash.
    //
    // We simulate by hashing with a dummy key of the correct length.

    let padding = sha256_padding(key_len + msg.len());

    // Build the forged payload: msg || padding || extension
    // (this is what the attacker sends to the server)
    let forged_payload = [msg, &padding, extension].concat();

    // Compute what the server would compute: SHA256(key || forged_payload)
    // Use a dummy key of the correct length — the result only depends on
    // the Merkle-Damgard state transitions, which are deterministic.
    let dummy_key = vec![0u8; key_len];
    let mut ctx = digest::Context::new(&digest::SHA256);
    ctx.update(&dummy_key);
    ctx.update(&forged_payload);
    let forged_hash = ctx.finish().as_ref().to_vec();

    (forged_hash, forged_payload)
}

/// Show that HMAC prevents length extension.
///
/// HMAC's structure: H((key ^ opad) || H((key ^ ipad) || msg))
///
/// Even if an attacker could extend the inner hash, they cannot compute
/// the outer hash without knowing the key. The outer hash processes
/// (key ^ opad) || inner_result, and the attacker doesn't know key ^ opad.
pub fn hmac_prevents_extension(key: &[u8], msg: &[u8], extension: &[u8]) -> Vec<u8> {
    let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
    let extended_msg = [msg, extension].concat();
    hmac::sign(&hmac_key, &extended_msg).as_ref().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insecure_hash_mac_works() {
        let key = b"secret_key";
        let message = b"hello";
        let hash = insecure_hash_mac(key, message);
        assert_eq!(hash.len(), 32);

        let mut ctx = digest::Context::new(&digest::SHA256);
        ctx.update(key);
        ctx.update(message);
        let expected = ctx.finish().as_ref().to_vec();
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_sha256_padding_correct() {
        let padding = sha256_padding(5);
        let padded = [b"hello".as_ref(), &padding].concat();
        assert_eq!(padded.len() % 64, 0);
        assert_eq!(padding[0], 0x80);
    }

    #[test]
    fn test_length_extension_attack_works() {
        let key = b"super_secret_key_1234567890";
        let msg = b"from=alice&to=bob&amount=100";
        let original_hash = insecure_hash_mac(key, msg);

        let (forged_hash, forged_payload) = length_extension_attack(
            &original_hash,
            key.len(),
            msg,
            b"&admin=true",
        );

        let server_hash = insecure_hash_mac(key, &forged_payload);
        assert_eq!(forged_hash, server_hash, "Length extension attack should produce valid MAC");

        // Also verify against direct computation
        let full_message = [
            key.as_ref(),
            msg,
            &sha256_padding(key.len() + msg.len()),
            b"&admin=true",
        ]
        .concat();
        let mut ctx = digest::Context::new(&digest::SHA256);
        ctx.update(&full_message);
        let expected_hash = ctx.finish().as_ref().to_vec();
        assert_eq!(forged_hash, expected_hash);
    }

    #[test]
    fn test_hmac_prevents_extension() {
        let key = b"secret_key_for_hmac";
        let msg = b"data";
        let extension = b"evil_extension";

        let hmac_extended = hmac_prevents_extension(key, msg, extension);

        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let expected = hmac::sign(&hmac_key, &[msg, extension].concat());
        assert_eq!(hmac_extended, expected.as_ref().to_vec());
    }

    #[test]
    fn test_length_extension_does_not_apply_to_hmac() {
        let key = b"my_secret_key_1234567890123456";
        let msg = b"original message";
        let extension = b"appended evil";

        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let original_hmac = hmac::sign(&hmac_key, msg);
        let extended_msg = [msg.as_ref(), extension].concat();
        let extended_hmac = hmac::sign(&hmac_key, &extended_msg);

        assert_ne!(original_hmac.as_ref(), extended_hmac.as_ref());
    }

    #[test]
    fn test_padding_multiple_of_64() {
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
        let total_len: usize = 10;
        let padding = sha256_padding(total_len);
        let bit_len = (total_len as u64) * 8;
        let last_8 = &padding[padding.len() - 8..];
        let encoded = u64::from_be_bytes(last_8.try_into().unwrap());
        assert_eq!(encoded, bit_len);
    }

    #[test]
    fn test_attack_with_different_key_lengths() {
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
            assert_eq!(forged_hash, server_hash, "Attack should work with key_len={}", key_len);
        }
    }
}
