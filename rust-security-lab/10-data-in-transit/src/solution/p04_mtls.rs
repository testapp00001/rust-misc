//! # Lesson 04: Mutual TLS (mTLS) (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TlsIdentity {
    pub common_name: String,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub issuer: String,
    pub is_ca: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MtlsState {
    Initial,
    ServerHelloSent,
    ClientCertSent,
    ServerVerified,
    ClientVerified,
    Connected,
    Failed(String),
}

pub struct MtlsConnection {
    pub state: MtlsState,
    pub server_identity: TlsIdentity,
    pub client_identity: Option<TlsIdentity>,
    pub trusted_cas: Vec<TlsIdentity>,
    pub transcript: Vec<u8>,
}

impl MtlsConnection {
    pub fn new_server(server_identity: TlsIdentity, trusted_cas: Vec<TlsIdentity>) -> Self {
        MtlsConnection {
            state: MtlsState::Initial,
            server_identity,
            client_identity: None,
            trusted_cas,
            transcript: Vec::new(),
        }
    }

    pub fn new_client(client_identity: TlsIdentity, trusted_cas: Vec<TlsIdentity>) -> Self {
        MtlsConnection {
            state: MtlsState::Initial,
            server_identity: TlsIdentity {
                common_name: String::new(),
                public_key: Vec::new(),
                private_key: Vec::new(),
                issuer: String::new(),
                is_ca: false,
            },
            client_identity: Some(client_identity),
            trusted_cas,
            transcript: Vec::new(),
        }
    }

    pub fn server_send_certificate(&mut self) -> Vec<u8> {
        let cert = serde_json::json!({
            "cn": self.server_identity.common_name,
            "public_key": self.server_identity.public_key,
            "issuer": self.server_identity.issuer,
            "is_ca": self.server_identity.is_ca,
        });
        let bytes = serde_json::to_vec(&cert).unwrap();
        self.transcript.extend_from_slice(&bytes);
        self.state = MtlsState::ServerHelloSent;
        bytes
    }

    pub fn server_request_client_cert(&self) -> Vec<u8> {
        let ca_names: Vec<&str> = self
            .trusted_cas
            .iter()
            .map(|ca| ca.common_name.as_str())
            .collect();
        let request = serde_json::json!({
            "type": "CertificateRequest",
            "acceptable_cas": ca_names,
        });
        serde_json::to_vec(&request).unwrap()
    }

    pub fn client_verify_server(&mut self, server_cert: &[u8]) -> Result<(), String> {
        let cert: serde_json::Value =
            serde_json::from_slice(server_cert).map_err(|e| format!("Invalid cert format: {}", e))?;

        let issuer = cert["issuer"]
            .as_str()
            .ok_or("Missing issuer in server cert")?;

        let is_trusted = self
            .trusted_cas
            .iter()
            .any(|ca| ca.common_name == issuer);

        if !is_trusted {
            return Err(format!("Server cert issuer '{}' is not trusted", issuer));
        }

        // Store the server identity for transcript
        self.server_identity = TlsIdentity {
            common_name: cert["cn"].as_str().unwrap_or("").to_string(),
            public_key: cert["public_key"]
                .as_array()
                .map(|a| a.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect())
                .unwrap_or_default(),
            private_key: Vec::new(),
            issuer: issuer.to_string(),
            is_ca: cert["is_ca"].as_bool().unwrap_or(false),
        };

        self.transcript.extend_from_slice(server_cert);
        self.state = MtlsState::ServerVerified;
        Ok(())
    }

    pub fn client_send_certificate(&self) -> Vec<u8> {
        let identity = self.client_identity.as_ref().expect("No client identity");
        let cert = serde_json::json!({
            "cn": identity.common_name,
            "public_key": identity.public_key,
            "issuer": identity.issuer,
            "is_ca": identity.is_ca,
        });
        serde_json::to_vec(&cert).unwrap()
    }

    pub fn server_verify_client(&mut self, client_cert: &[u8]) -> Result<(), String> {
        let cert: serde_json::Value =
            serde_json::from_slice(client_cert).map_err(|e| format!("Invalid cert format: {}", e))?;

        let issuer = cert["issuer"]
            .as_str()
            .ok_or("Missing issuer in client cert")?;

        let is_trusted = self
            .trusted_cas
            .iter()
            .any(|ca| ca.common_name == issuer);

        if !is_trusted {
            return Err(format!("Client cert issuer '{}' is not trusted", issuer));
        }

        self.client_identity = Some(TlsIdentity {
            common_name: cert["cn"].as_str().unwrap_or("").to_string(),
            public_key: cert["public_key"]
                .as_array()
                .map(|a| a.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect())
                .unwrap_or_default(),
            private_key: Vec::new(),
            issuer: issuer.to_string(),
            is_ca: cert["is_ca"].as_bool().unwrap_or(false),
        });

        self.transcript.extend_from_slice(client_cert);
        self.state = MtlsState::ClientVerified;
        Ok(())
    }

    pub fn complete_handshake(&mut self) -> Result<(), String> {
        if self.state != MtlsState::ClientVerified && self.state != MtlsState::ServerVerified {
            return Err(format!(
                "Cannot complete handshake: state is {:?}",
                self.state
            ));
        }
        // For server: need both verified. For client: need server verified at minimum.
        // In a full impl both sides need ClientVerified AND ServerVerified.
        self.state = MtlsState::Connected;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.state == MtlsState::Connected
    }
}

pub fn create_ca(common_name: &str) -> TlsIdentity {
    let rng = SystemRandom::new();
    let mut public_key = vec![0u8; 32];
    rng.fill(&mut public_key).unwrap();
    let private_key: Vec<u8> = public_key.iter().map(|b| b.wrapping_add(128)).collect();

    TlsIdentity {
        common_name: common_name.to_string(),
        public_key,
        private_key,
        issuer: common_name.to_string(),
        is_ca: true,
    }
}

pub fn issue_certificate(ca: &TlsIdentity, common_name: &str, public_key: Vec<u8>) -> TlsIdentity {
    // In a real system, this would create a proper X.509 cert signed by the CA.
    // Here we just set the issuer and trust the public key directly.
    TlsIdentity {
        common_name: common_name.to_string(),
        public_key,
        private_key: Vec::new(), // End-entity doesn't need private key in this model
        issuer: ca.common_name.clone(),
        is_ca: false,
    }
}

pub fn verify_identity(identity: &TlsIdentity, trusted_cas: &[TlsIdentity]) -> bool {
    trusted_cas.iter().any(|ca| ca.common_name == identity.issuer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_identity(cn: &str, key: u8, issuer: &str, is_ca: bool) -> TlsIdentity {
        TlsIdentity {
            common_name: cn.to_string(),
            public_key: vec![key; 32],
            private_key: vec![key + 100; 32],
            issuer: issuer.to_string(),
            is_ca,
        }
    }

    #[test]
    fn test_create_ca() {
        let ca = create_ca("Test CA");
        assert!(ca.is_ca);
        assert_eq!(ca.common_name, "Test CA");
    }

    #[test]
    fn test_issue_certificate() {
        let ca = create_ca("Test CA");
        let cert = issue_certificate(&ca, "server.example.com", vec![42u8; 32]);
        assert!(!cert.is_ca);
        assert_eq!(cert.issuer, "Test CA");
    }

    #[test]
    fn test_mtls_server_init() {
        let server = make_identity("server.com", 1, "CA", false);
        let ca = make_identity("CA", 2, "CA", true);
        let conn = MtlsConnection::new_server(server.clone(), vec![ca]);
        assert_eq!(conn.state, MtlsState::Initial);
        assert!(conn.client_identity.is_none());
    }

    #[test]
    fn test_mtls_full_handshake() {
        let ca = create_ca("Test CA");
        let server_id = issue_certificate(&ca, "server.com", vec![1u8; 32]);
        let client_id = issue_certificate(&ca, "client.com", vec![2u8; 32]);

        let mut server_conn = MtlsConnection::new_server(server_id.clone(), vec![ca.clone()]);
        let mut client_conn = MtlsConnection::new_client(client_id.clone(), vec![ca.clone()]);

        let server_cert = server_conn.server_send_certificate();
        client_conn.client_verify_server(&server_cert).unwrap();

        let client_cert = client_conn.client_send_certificate();
        server_conn.server_verify_client(&client_cert).unwrap();

        server_conn.complete_handshake().unwrap();
        client_conn.complete_handshake().unwrap();

        assert!(server_conn.is_connected());
        assert!(client_conn.is_connected());
    }

    #[test]
    fn test_mtls_rejects_untrusted_client() {
        let ca = create_ca("Trusted CA");
        let rogue_ca = create_ca("Rogue CA");
        let server_id = issue_certificate(&ca, "server.com", vec![1u8; 32]);
        let rogue_client = issue_certificate(&rogue_ca, "evil.com", vec![2u8; 32]);

        let mut server_conn = MtlsConnection::new_server(server_id, vec![ca]);
        let client_cert_bytes = serde_json::to_vec(&serde_json::json!({
            "cn": rogue_client.common_name,
            "public_key": rogue_client.public_key,
            "issuer": rogue_client.issuer,
        }))
        .unwrap();

        let result = server_conn.server_verify_client(&client_cert_bytes);
        assert!(result.is_err(), "Should reject client from untrusted CA");
    }

    #[test]
    fn test_verify_identity_valid() {
        let ca = create_ca("Test CA");
        let cert = issue_certificate(&ca, "server.com", vec![42u8; 32]);
        assert!(verify_identity(&cert, &[ca]));
    }

    #[test]
    fn test_verify_identity_untrusted() {
        let ca = create_ca("Test CA");
        let rogue_ca = create_ca("Rogue CA");
        let cert = issue_certificate(&rogue_ca, "evil.com", vec![42u8; 32]);
        assert!(!verify_identity(&cert, &[ca]));
    }

    #[test]
    fn test_mtls_cannot_connect_without_both_verified() {
        let ca = create_ca("Test CA");
        let server_id = issue_certificate(&ca, "server.com", vec![1u8; 32]);
        let mut server_conn = MtlsConnection::new_server(server_id, vec![ca]);
        let result = server_conn.complete_handshake();
        assert!(result.is_err(), "Cannot complete without client verification");
    }
}
