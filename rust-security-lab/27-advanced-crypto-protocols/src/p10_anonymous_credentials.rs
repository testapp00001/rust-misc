//! # Lesson 10: Anonymous Credentials -- Prove Attributes Without Revealing Identity
//!
//! ## What are Anonymous Credentials?
//!
//! Anonymous credentials allow a user to prove statements about their attributes
//! (e.g., "I am over 18", "I am a member of group X") without revealing which specific
//! credential they hold or their identity.
//!
//! ## How It Works
//!
//! 1. **Issuance**: An issuer gives the user a credential (a signed statement of attributes).
//!    The user's identity is known to the issuer at this point.
//!
//! 2. **Presentation**: The user proves a predicate over their attributes in zero-knowledge:
//!    - "age >= 18" without revealing exact age
//!    - "department == engineering" without revealing employee ID
//!    - "has_valid_subscription" without revealing which account
//!
//! 3. **Verification**: The verifier checks the proof against the issuer's public key.
//!    They learn only the proven predicate, nothing else.
//!
//! ## Camenisch-Lysyanskaya (CL) Signatures
//!
//! The standard construction uses CL signatures:
//! - Issuer signs a vector of attributes (a1, a2, ..., an)
//! - User proves knowledge of a valid signature on attributes satisfying the predicate
//! - Uses pairings or RSA-based commitments
//!
//! ## Simplified Model
//!
//! In this exercise we use a simplified model:
//! - Credentials are hash-based (H(attributes || issuer_secret))
//! - Proofs use commitment-and-reveal for non-sensitive attributes
//! - Zero-knowledge is simulated using hash commitments
//!
//! ## Attack: Credential Sharing
//!
//! If a credential can be copied, users can share credentials. Prevented by:
//! - Binding the credential to a user-specific secret (e.g., private key)
//! - Using "one-show" credentials that become linkable if reused
//!
//! ## Attack: Issuer Impersonation
//!
//! If the issuer's public key is not authenticated, an adversary can issue fake credentials.
//! Requires a PKI or trust anchor for issuer keys.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// An issuer's public key (for verifying credentials).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerPublicKey {
    /// Unique identifier for the issuer
    pub issuer_id: String,
    /// Public verification key (hash of the signing secret)
    pub verification_key: Vec<u8>,
}

/// An issuer's private key (for issuing credentials).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerPrivateKey {
    /// The signing secret
    pub signing_secret: Vec<u8>,
    /// Corresponding public key
    pub public_key: IssuerPublicKey,
}

/// A credential issued to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// The attributes certified by the issuer
    pub attributes: Vec<Attribute>,
    /// The issuer's signature on the attributes
    pub signature: Vec<u8>,
    /// The issuer's public key (for verification)
    pub issuer_pk: IssuerPublicKey,
}

/// A user attribute (name-value pair).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

/// Possible attribute values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}

/// A zero-knowledge presentation proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationProof {
    /// Revealed attributes (those the user chooses to show)
    pub revealed: Vec<Attribute>,
    /// Commitment to hidden attributes
    pub hidden_commitment: Vec<u8>,
    /// ZK proof that hidden attributes satisfy the predicate
    pub predicate_proof: Vec<u8>,
    /// Nonce to prevent replay
    pub nonce: Vec<u8>,
}

/// A predicate over attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Predicate {
    /// Attribute equals a specific value
    Equals(String, AttributeValue),
    /// Integer attribute is at least a value
    GreaterOrEqual(String, i64),
    /// Integer attribute is at most a value
    LessOrEqual(String, i64),
    /// Boolean attribute is true
    IsTrue(String),
    /// Attribute exists
    Exists(String),
}

/// Exercise 1: Generate an issuer key pair.
///
/// Generate a random signing secret (32 bytes) and derive the public
/// verification key as SHA-256(signing_secret).
pub fn generate_issuer_key(issuer_id: &str) -> (IssuerPublicKey, IssuerPrivateKey) {
    todo!("Generate issuer key pair")
}

/// Exercise 2: Issue a credential.
///
/// The issuer signs the attributes:
/// 1. Serialize attributes to bytes
/// 2. Compute signature = SHA-256(attribute_bytes || signing_secret)
/// 3. Return the Credential
pub fn issue_credential(
    attributes: &[Attribute],
    issuer_sk: &IssuerPrivateKey,
) -> Credential {
    todo!("Issue a signed credential with the given attributes")
}

/// Exercise 3: Verify a credential's signature.
///
/// Recompute the signature and check it matches.
pub fn verify_credential(credential: &Credential) -> bool {
    todo!("Verify that a credential has a valid issuer signature")
}

/// Exercise 4: Create a presentation proof.
///
/// The user proves a predicate over their attributes:
/// 1. Choose which attributes to reveal and which to hide
/// 2. Compute commitment to hidden attributes
/// 3. Create a proof that hidden attributes satisfy the predicate
/// 4. Include a nonce for freshness
///
/// The proof demonstrates knowledge of a valid credential with attributes
/// satisfying the predicate, without revealing hidden attributes.
pub fn create_presentation(
    credential: &Credential,
    predicates: &[Predicate],
    reveal_names: &[String],
    nonce: Vec<u8>,
) -> PresentationProof {
    todo!("Create a ZK presentation proof")
}

/// Exercise 5: Verify a presentation proof.
///
/// Check that:
/// 1. Revealed attributes satisfy all predicates they are involved in
/// 2. The predicate proof is valid (proves hidden attributes satisfy remaining predicates)
/// 3. The credential signature is consistent with the proof
/// 4. The nonce is fresh
pub fn verify_presentation(
    proof: &PresentationProof,
    issuer_pk: &IssuerPublicKey,
    predicates: &[Predicate],
) -> Result<(), String> {
    todo!("Verify a presentation proof against predicates")
}

/// Exercise 6: Check if a single predicate is satisfied by attributes.
///
/// Evaluate a predicate against a set of attributes.
pub fn check_predicate(predicate: &Predicate, attributes: &[Attribute]) -> bool {
    todo!("Evaluate a predicate against attributes")
}

/// Exercise 7: Prevent credential sharing (linkability check).
///
/// In a real system, this would use a user-specific binding (e.g., a private key
/// embedded in the credential). Here we simulate by checking that the presentation
/// includes a user-specific proof that cannot be forged.
///
/// Return true if the presentation is bound to the expected user.
pub fn check_user_binding(
    proof: &PresentationProof,
    user_secret: &[u8],
) -> bool {
    todo!("Check that a presentation is bound to a specific user")
}

/// Helper: hash data with SHA-256.
pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Helper: serialize attributes to bytes.
pub fn serialize_attributes(attributes: &[Attribute]) -> Vec<u8> {
    let mut data = Vec::new();
    for attr in attributes {
        data.extend_from_slice(attr.name.as_bytes());
        match &attr.value {
            AttributeValue::String(s) => {
                data.push(0);
                data.extend_from_slice(s.as_bytes());
            }
            AttributeValue::Integer(i) => {
                data.push(1);
                data.extend_from_slice(&i.to_le_bytes());
            }
            AttributeValue::Boolean(b) => {
                data.push(2);
                data.push(if *b { 1 } else { 0 });
            }
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_issuer() -> (IssuerPublicKey, IssuerPrivateKey) {
        generate_issuer_key("TestIssuer")
    }

    fn test_attributes() -> Vec<Attribute> {
        vec![
            Attribute { name: "name".into(), value: AttributeValue::String("Alice".into()) },
            Attribute { name: "age".into(), value: AttributeValue::Integer(25) },
            Attribute { name: "citizen".into(), value: AttributeValue::Boolean(true) },
        ]
    }

    #[test]
    fn test_generate_issuer_key() {
        let (pk, sk) = test_issuer();
        assert_eq!(pk.issuer_id, "TestIssuer");
        assert!(!pk.verification_key.is_empty());
        assert!(!sk.signing_secret.is_empty());
    }

    #[test]
    fn test_issue_and_verify() {
        let (pk, sk) = test_issuer();
        let attrs = test_attributes();
        let cred = issue_credential(&attrs, &sk);
        assert!(verify_credential(&cred));
    }

    #[test]
    fn test_credential_contains_attributes() {
        let (_, sk) = test_issuer();
        let attrs = test_attributes();
        let cred = issue_credential(&attrs, &sk);
        assert_eq!(cred.attributes, attrs);
    }

    #[test]
    fn test_tampered_credential_fails() {
        let (_, sk) = test_issuer();
        let attrs = test_attributes();
        let mut cred = issue_credential(&attrs, &sk);

        // Save the original signature, then tamper
        let original_sig = cred.signature.clone();
        cred.attributes[1].value = AttributeValue::Integer(99);

        // Re-issue with tampered attributes produces a different signature
        let tampered_cred = issue_credential(&cred.attributes, &sk);
        assert_ne!(original_sig, tampered_cred.signature,
            "Signature should change when attributes are tampered");
    }

    #[test]
    fn test_predicate_equals() {
        let attrs = test_attributes();
        let pred = Predicate::Equals("name".into(), AttributeValue::String("Alice".into()));
        assert!(check_predicate(&pred, &attrs));

        let pred_wrong = Predicate::Equals("name".into(), AttributeValue::String("Bob".into()));
        assert!(!check_predicate(&pred_wrong, &attrs));
    }

    #[test]
    fn test_predicate_greater_or_equal() {
        let attrs = test_attributes();
        let pred = Predicate::GreaterOrEqual("age".into(), 18);
        assert!(check_predicate(&pred, &attrs));

        let pred_fail = Predicate::GreaterOrEqual("age".into(), 30);
        assert!(!check_predicate(&pred_fail, &attrs));
    }

    #[test]
    fn test_predicate_is_true() {
        let attrs = test_attributes();
        let pred = Predicate::IsTrue("citizen".into());
        assert!(check_predicate(&pred, &attrs));
    }

    #[test]
    fn test_presentation_proof() {
        let (pk, sk) = test_issuer();
        let attrs = test_attributes();
        let cred = issue_credential(&attrs, &sk);

        // Prove age >= 18, reveal only "citizen"
        let predicates = vec![Predicate::GreaterOrEqual("age".into(), 18)];
        let reveal = vec!["citizen".to_string()];
        let nonce = vec![1, 2, 3, 4];

        let proof = create_presentation(&cred, &predicates, &reveal, nonce);
        // Should have revealed "citizen"
        assert!(proof.revealed.iter().any(|a| a.name == "citizen"));
        // Should NOT have revealed "age" or "name" directly
        assert!(!proof.revealed.iter().any(|a| a.name == "age"));
    }

    #[test]
    fn test_verify_presentation() {
        let (pk, sk) = test_issuer();
        let attrs = test_attributes();
        let cred = issue_credential(&attrs, &sk);

        let predicates = vec![Predicate::GreaterOrEqual("age".into(), 18)];
        let reveal = vec!["citizen".to_string()];
        let nonce = vec![1, 2, 3, 4];

        let proof = create_presentation(&cred, &predicates, &reveal, nonce);
        assert!(verify_presentation(&proof, &pk, &predicates).is_ok());
    }

    #[test]
    fn test_check_user_binding() {
        let user_secret = b"alice_private_key";
        let (pk, sk) = test_issuer();
        let attrs = test_attributes();
        let cred = issue_credential(&attrs, &sk);
        let predicates = vec![Predicate::IsTrue("citizen".into())];
        let proof = create_presentation(&cred, &predicates, &[], vec![42]);

        assert!(check_user_binding(&proof, user_secret));
    }
}
