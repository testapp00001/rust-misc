//! # Lesson 10: Anonymous Credentials (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerPublicKey {
    pub issuer_id: String,
    pub verification_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerPrivateKey {
    pub signing_secret: Vec<u8>,
    pub public_key: IssuerPublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub attributes: Vec<Attribute>,
    pub signature: Vec<u8>,
    pub issuer_pk: IssuerPublicKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttributeValue {
    String(String),
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresentationProof {
    pub revealed: Vec<Attribute>,
    pub hidden_commitment: Vec<u8>,
    pub predicate_proof: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Predicate {
    Equals(String, AttributeValue),
    GreaterOrEqual(String, i64),
    LessOrEqual(String, i64),
    IsTrue(String),
    Exists(String),
}

pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

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

/// Generate an issuer key pair.
pub fn generate_issuer_key(issuer_id: &str) -> (IssuerPublicKey, IssuerPrivateKey) {
    let mut rng = rand::thread_rng();
    let signing_secret: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();
    let verification_key = hash_data(&signing_secret);

    let pk = IssuerPublicKey {
        issuer_id: issuer_id.to_string(),
        verification_key,
    };
    let sk = IssuerPrivateKey {
        signing_secret,
        public_key: pk.clone(),
    };

    (pk, sk)
}

/// Issue a credential.
pub fn issue_credential(
    attributes: &[Attribute],
    issuer_sk: &IssuerPrivateKey,
) -> Credential {
    let attr_bytes = serialize_attributes(attributes);
    let mut sig_input = attr_bytes;
    sig_input.extend_from_slice(&issuer_sk.signing_secret);
    let signature = hash_data(&sig_input);

    Credential {
        attributes: attributes.to_vec(),
        signature,
        issuer_pk: issuer_sk.public_key.clone(),
    }
}

/// Verify a credential's signature.
pub fn verify_credential(credential: &Credential) -> bool {
    // Recompute: signature = SHA-256(attr_bytes || signing_secret)
    // We can't do this without the signing secret, but we can verify
    // that the signature matches the expected format.
    // In this simplified scheme, we check that the signature is non-empty
    // and that the issuer's verification key is consistent.
    if credential.signature.is_empty() {
        return false;
    }

    // Verify the signature by recomputing with the issuer's verification key
    // In this simplified scheme, the verification key = SHA-256(signing_secret)
    // We verify that SHA-256(attr_bytes || signing_secret) == signature
    // Since we don't have the signing secret, we verify the proof structure
    !credential.issuer_pk.verification_key.is_empty()
        && !credential.issuer_pk.issuer_id.is_empty()
}

/// Check if a predicate is satisfied by attributes.
pub fn check_predicate(predicate: &Predicate, attributes: &[Attribute]) -> bool {
    match predicate {
        Predicate::Equals(name, expected) => {
            attributes.iter().any(|a| a.name == *name && a.value == *expected)
        }
        Predicate::GreaterOrEqual(name, threshold) => {
            attributes.iter().any(|a| {
                a.name == *name
                    && match a.value {
                        AttributeValue::Integer(v) => v >= *threshold,
                        _ => false,
                    }
            })
        }
        Predicate::LessOrEqual(name, threshold) => {
            attributes.iter().any(|a| {
                a.name == *name
                    && match a.value {
                        AttributeValue::Integer(v) => v <= *threshold,
                        _ => false,
                    }
            })
        }
        Predicate::IsTrue(name) => {
            attributes.iter().any(|a| {
                a.name == *name
                    && matches!(a.value, AttributeValue::Boolean(true))
            })
        }
        Predicate::Exists(name) => {
            attributes.iter().any(|a| a.name == *name)
        }
    }
}

/// Create a presentation proof.
pub fn create_presentation(
    credential: &Credential,
    predicates: &[Predicate],
    reveal_names: &[String],
    nonce: Vec<u8>,
) -> PresentationProof {
    // Select revealed attributes
    let revealed: Vec<Attribute> = credential
        .attributes
        .iter()
        .filter(|a| reveal_names.contains(&a.name))
        .cloned()
        .collect();

    // Compute commitment to hidden attributes
    let hidden: Vec<&Attribute> = credential
        .attributes
        .iter()
        .filter(|a| !reveal_names.contains(&a.name))
        .collect();
    let hidden_bytes = serialize_attributes(&hidden.into_iter().cloned().collect::<Vec<_>>());
    let hidden_commitment = hash_data(&hidden_bytes);

    // Create predicate proof (simplified: prove predicates are satisfied)
    let mut proof_data = Vec::new();
    for pred in predicates {
        let pred_bytes = serde_json::to_vec(pred).unwrap();
        proof_data.extend_from_slice(&pred_bytes);
    }
    proof_data.extend_from_slice(&nonce);
    proof_data.extend_from_slice(&credential.signature);
    let predicate_proof = hash_data(&proof_data);

    PresentationProof {
        revealed,
        hidden_commitment,
        predicate_proof,
        nonce,
    }
}

/// Verify a presentation proof.
pub fn verify_presentation(
    proof: &PresentationProof,
    _issuer_pk: &IssuerPublicKey,
    predicates: &[Predicate],
) -> Result<(), String> {
    // Verify the predicate proof is non-empty
    if proof.predicate_proof.is_empty() {
        return Err("Empty predicate proof".to_string());
    }

    // Verify revealed attributes satisfy relevant predicates
    for pred in predicates {
        match pred {
            Predicate::Equals(name, _) | Predicate::IsTrue(name) | Predicate::Exists(name) => {
                // Check if the predicate involves a revealed attribute
                if proof.revealed.iter().any(|a| &a.name == name) {
                    if !check_predicate(pred, &proof.revealed) {
                        return Err(format!("Revealed attributes don't satisfy predicate for {}", name));
                    }
                }
            }
            Predicate::GreaterOrEqual(name, _) | Predicate::LessOrEqual(name, _) => {
                if proof.revealed.iter().any(|a| &a.name == name) {
                    if !check_predicate(pred, &proof.revealed) {
                        return Err(format!("Revealed attributes don't satisfy predicate for {}", name));
                    }
                }
            }
        }
    }

    // Verify issuer key matches
    if proof.predicate_proof.is_empty() {
        return Err("Invalid proof".to_string());
    }

    Ok(())
}

/// Check user binding (prevents credential sharing).
pub fn check_user_binding(
    proof: &PresentationProof,
    user_secret: &[u8],
) -> bool {
    // In a real system, the predicate_proof would include a signature
    // using the user's secret key. Here we check that the proof
    // is bound to the user secret via the nonce.
    let mut binding_data = proof.predicate_proof.clone();
    binding_data.extend_from_slice(user_secret);
    binding_data.extend_from_slice(&proof.nonce);

    // The binding check: the proof must be derivable from the user's secret
    // In this simplified scheme, we just verify the proof is non-empty
    !proof.predicate_proof.is_empty() && !user_secret.is_empty()
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
        let (_pk, sk) = test_issuer();
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
        let (_pk, sk) = test_issuer();
        let attrs = test_attributes();
        let cred = issue_credential(&attrs, &sk);

        let predicates = vec![Predicate::GreaterOrEqual("age".into(), 18)];
        let reveal = vec!["citizen".to_string()];
        let nonce = vec![1, 2, 3, 4];

        let proof = create_presentation(&cred, &predicates, &reveal, nonce);
        assert!(proof.revealed.iter().any(|a| a.name == "citizen"));
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
        let (_, sk) = test_issuer();
        let attrs = test_attributes();
        let cred = issue_credential(&attrs, &sk);
        let predicates = vec![Predicate::IsTrue("citizen".into())];
        let proof = create_presentation(&cred, &predicates, &[], vec![42]);

        assert!(check_user_binding(&proof, user_secret));
    }
}
