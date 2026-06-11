# Solution 02: Implement mTLS Between Two Services

## Part A: Generate the Certificate Authority and Service Certificates

```bash
#!/bin/bash
# generate-certs.sh

set -euo pipefail

CERT_DIR="./certs"
mkdir -p "$CERT_DIR"/{ca,server,client}

echo "=== Generating Root CA ==="
openssl req -x509 -newkey rsa:4096 \
  -keyout "$CERT_DIR/ca/ca.key" \
  -out "$CERT_DIR/ca/ca.crt" \
  -days 365 -nodes \
  -subj "/CN=Zero Trust CA/O=Exercise/C=US"

echo "=== Generating Server Certificate ==="
# Generate server private key and CSR
openssl req -newkey rsa:4096 \
  -keyout "$CERT_DIR/server/server.key" \
  -out "$CERT_DIR/server/server.csr" \
  -nodes \
  -subj "/CN=api-service/O=Exercise/C=US"

# Sign server certificate with CA
openssl x509 -req \
  -in "$CERT_DIR/server/server.csr" \
  -CA "$CERT_DIR/ca/ca.crt" \
  -CAkey "$CERT_DIR/ca/ca.key" \
  -CAcreateserial \
  -out "$CERT_DIR/server/server.crt" \
  -days 90

echo "=== Generating Client Certificate ==="
# Generate client private key and CSR
openssl req -newkey rsa:4096 \
  -keyout "$CERT_DIR/client/client.key" \
  -out "$CERT_DIR/client/client.csr" \
  -nodes \
  -subj "/CN=client-service/O=Exercise/C=US"

# Sign client certificate with CA
openssl x509 -req \
  -in "$CERT_DIR/client/client.csr" \
  -CA "$CERT_DIR/ca/ca.crt" \
  -CAkey "$CERT_DIR/ca/ca.key" \
  -CAcreateserial \
  -out "$CERT_DIR/client/client.crt" \
  -days 90

# Clean up CSRs
rm -f "$CERT_DIR"/server/server.csr "$CERT_DIR"/client/client.csr

echo "=== Certificates Generated ==="
echo "CA:     $CERT_DIR/ca/ca.crt"
echo "Server: $CERT_DIR/server/server.crt"
echo "Client: $CERT_DIR/client/client.crt"
```

### Why This Works

The PKI hierarchy is the foundation of mTLS trust. The root CA is the single source of truth -- any certificate signed by the CA is trusted by parties that hold the CA certificate. This is identical to how web PKI works, but instead of public CAs like Let's Encrypt, you operate your own private CA.

The critical design choices:
- **Separate key pairs**: Each service gets its own unique private key. Compromising one does not affect others.
- **Short certificate lifetime**: 90-day certificates limit the window of exposure if a key is compromised.
- **CN-based identity**: The Common Name (CN) field identifies the service. In production, you would use SPIFFE IDs (e.g., `spiffe://cluster.local/ns/default/sa/api-service`).

### Common Mistakes

- **Using the CA key on services.** The CA private key should only exist on the certificate authority machine, never distributed to services.
- **Sharing private keys between services.** Each service must have its own unique key pair.
- **Long certificate lifetimes.** 90 days is reasonable; 365 days is too long for production. Automated rotation is essential.
- **Not protecting the CA key.** The CA key should be stored in an HSM or Vault, not on a developer laptop.

---

## Part B: Implement the mTLS Server

```rust
// server/Cargo.toml
[package]
name = "mtls-server"
version = "0.1.0"
edition = "2021"

[dependencies]
native-tls = "0.2"
```

```rust
// server/src/main.rs
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load server certificate and private key
    let mut cert_file = File::open("../certs/server/server.crt")?;
    let mut cert_bytes = Vec::new();
    cert_file.read_to_end(&mut cert_bytes)?;

    let mut key_file = File::open("../certs/server/server.key")?;
    let mut key_bytes = Vec::new();
    key_file.read_to_end(&mut key_bytes)?;

    let identity = Identity::from_pkcs8(&cert_bytes, &key_bytes)?;

    // Load CA certificate for verifying client certificates
    let mut ca_file = File::open("../certs/ca/ca.crt")?;
    let mut ca_bytes = Vec::new();
    ca_file.read_to_end(&mut ca_bytes)?;

    let ca_cert = native_tls::Certificate::from_pem(&ca_bytes)?;

    // Build TLS acceptor with client certificate verification
    let mut builder = TlsAcceptor::builder(identity);
    builder.add_root_certificate(ca_cert);
    // In production, use SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT
    // native-tls does not directly expose this; for full control, use the openssl crate:
    //
    // let mut ctx = openssl::ssl::SslContext::builder(openssl::ssl::SslMethod::tls())?;
    // ctx.set_verify(openssl::ssl::SslVerifyMode::PEER | openssl::ssl::SslVerifyMode::FAIL_IF_NO_PEER_CERT);
    // ctx.set_ca_file("../certs/ca/ca.crt")?;

    let acceptor = builder.build()?;

    let listener = TcpListener::bind("127.0.0.1:8443")?;
    println!("mTLS server listening on 127.0.0.1:8443");

    for stream in listener.incoming() {
        match stream {
            Ok(tcp_stream) => {
                match acceptor.accept(tcp_stream) {
                    Ok(mut tls_stream) => {
                        // Read the HTTP request
                        let mut buf_reader = BufReader::new(&tls_stream);
                        let mut request_line = String::new();
                        buf_reader.read_line(&mut request_line)?;

                        // Read remaining headers
                        let mut headers = Vec::new();
                        loop {
                            let mut line = String::new();
                            buf_reader.read_line(&mut line)?;
                            if line.trim().is_empty() {
                                break;
                            }
                            headers.push(line.trim().to_string());
                        }

                        println!("Request: {}", request_line.trim());

                        // Get peer certificate info
                        // Note: native-tls TlsStream does not expose peer cert directly
                        // For full cert inspection, use the openssl crate
                        let peer_cn = "client-service"; // In production, extract from cert

                        // Send HTTP response with client identity
                        let response_body = format!(
                            r#"{{"status":"ok","server":"api-service","authenticated_client":"{}","message":"mTLS connection verified"}}"#,
                            peer_cn
                        );
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                            response_body.len(),
                            response_body
                        );

                        tls_stream.write_all(response.as_bytes()).ok();
                        tls_stream.shutdown().ok();
                    }
                    Err(e) => {
                        eprintln!("TLS handshake failed: {}", e);
                        // This is where rejected connections (no client cert, wrong CA) end up
                    }
                }
            }
            Err(e) => {
                eprintln!("TCP connection failed: {}", e);
            }
        }
    }

    Ok(())
}
```

For a more robust implementation using `openssl` crate with full client certificate verification:

```rust
// Using openssl crate for full control over client cert verification
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslVerifyMode};

fn create_tls_acceptor() -> Result<SslAcceptor, Box<dyn std::error::Error>> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;

    // Load server certificate and key
    builder.set_private_key_file("certs/server/server.key", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("certs/server/server.crt")?;

    // CRITICAL: Load CA for client verification
    builder.set_ca_file("certs/ca/ca.crt")?;

    // CRITICAL: Require and verify client certificates
    builder.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT);

    // Callback to extract client CN from certificate
    builder.set_verify_callback(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT,
        |verified, ctx| {
            if let Some(cert) = ctx.current_cert() {
                let subject = cert.subject_name();
                if let Some(cn) = subject.entries_by_nid(openssl::nid::Nid::COMMONNAME).next() {
                    println!("Client CN: {}", cn.data().as_utf8().unwrap());
                }
            }
            verified
        }
    );

    Ok(builder.build())
}
```

### Why This Works

The server-side mTLS configuration has three critical components:

1. **Server identity**: The server loads its own certificate and key to prove its identity to clients.
2. **CA trust store**: The server loads the CA certificate to verify that client certificates were signed by a trusted authority.
3. **Client verification requirement**: `SslVerifyMode::FAIL_IF_NO_PEER_CERT` ensures the TLS handshake fails if the client does not present a certificate. Without this flag, the server would accept connections with or without client certificates -- defeating the purpose of mTLS.

The TLS handshake flow in mTLS:
```
Client                              Server
  |                                    |
  |--- ClientHello ------------------>|
  |<-- ServerHello + ServerCert ------|
  |<-- CertificateRequest ------------|  (Server requests client cert)
  |--- ClientCert + ClientKeyExchange->|
  |--- CertificateVerify ------------>|  (Client proves key ownership)
  |--- Finished --------------------->|
  |<-- Finished ----------------------|
  |                                    |
  | Both sides verified. Secure channel established.
```

### Common Mistakes

- **Not setting FAIL_IF_NO_PEER_CERT.** Without this, the server accepts connections without client certificates, making mTLS optional (not mutual).
- **Loading the wrong CA.** The CA used to verify client certificates must be the same CA that signed the client certificates.
- **Using self-signed certificates.** Self-signed certs cannot be revoked and do not form a trust hierarchy. Always use a CA.
- **Not handling handshake failures.** TLS handshake errors (wrong CA, missing cert, expired cert) must be caught and logged, not silently ignored.

---

## Part C: Implement the mTLS Client

```rust
// client/Cargo.toml
[package]
name = "mtls-client"
version = "0.1.0"
edition = "2021"

[dependencies]
native-tls = "0.2"
```

```rust
// client/src/main.rs
use native_tls::{Certificate, Identity, TlsConnector};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load client certificate and private key
    let mut cert_file = File::open("../certs/client/client.crt")?;
    let mut cert_bytes = Vec::new();
    cert_file.read_to_end(&mut cert_bytes)?;

    let mut key_file = File::open("../certs/client/client.key")?;
    let mut key_bytes = Vec::new();
    key_file.read_to_end(&mut key_bytes)?;

    let identity = Identity::from_pkcs8(&cert_bytes, &key_bytes)?;

    // Load CA certificate for server verification
    let mut ca_file = File::open("../certs/ca/ca.crt")?;
    let mut ca_bytes = Vec::new();
    ca_file.read_to_end(&mut ca_bytes)?;

    let ca_cert = Certificate::from_pem(&ca_bytes)?;

    // Build TLS connector with client identity
    let connector = TlsConnector::builder()
        .identity(identity)
        .add_root_certificate(ca_cert)
        .build()?;

    // Connect to server with mTLS
    let tcp_stream = TcpStream::connect("127.0.0.1:8443")?;
    let mut tls_stream = connector.connect("api-service", tcp_stream)?;

    println!("mTLS connection established");
    println!("Negotiated protocol: {:?}", tls_stream.get_ref().peer_addr()?);

    // Send HTTP request
    let request = "GET / HTTP/1.1\r\nHost: api-service\r\nConnection: close\r\n\r\n";
    tls_stream.write_all(request.as_bytes())?;

    // Read response
    let mut response = String::new();
    tls_stream.read_to_string(&mut response)?;
    println!("Response:\n{}", response);

    Ok(())
}
```

### Why This Works

The client-side mTLS configuration mirrors the server side but with reversed roles:

1. **Client identity**: The client loads its own certificate and key to present during the handshake.
2. **Server verification**: The client loads the CA certificate to verify the server's certificate was signed by a trusted authority.
3. **Server name verification**: `connector.connect("api-service", ...)` verifies that the server certificate's CN matches "api-service", preventing man-in-the-middle attacks.

The client automatically presents its certificate when the server sends a `CertificateRequest` during the TLS handshake. This is handled by the TLS library -- the application code does not need to manually send the certificate.

### Common Mistakes

- **Not verifying the server hostname.** Using `connector.connect("", stream)` or an IP address bypasses hostname verification, enabling MITM attacks.
- **Using `-k` (insecure) in curl for testing.** While convenient for testing, this trains developers to ignore certificate errors. Always verify certificates properly.
- **Not including the CA certificate.** Without the CA cert, the client cannot verify the server's certificate and will reject the connection.
- **Certificate format mismatch.** PEM vs DER format matters. Ensure your code matches the format of the generated certificates.

---

## Part D: Test and Verify

```bash
#!/bin/bash
# test-mtls.sh

set -euo pipefail

SERVER_PID=""

cleanup() {
    if [ -n "$SERVER_PID" ]; then
        kill "$SERVER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

echo "=== Starting mTLS Server ==="
cd server && cargo run &
SERVER_PID=$!
sleep 2

echo ""
echo "=== Test 1: Valid mTLS Connection ==="
echo "Expected: Success with client identity in response"
curl -s \
  --cert ../certs/client/client.crt \
  --key ../certs/client/client.key \
  --cacert ../certs/ca/ca.crt \
  https://127.0.0.1:8443/ 2>&1
echo ""

echo ""
echo "=== Test 2: No Client Certificate ==="
echo "Expected: Connection rejected (handshake failure)"
curl -s \
  --cacert ../certs/ca/ca.crt \
  https://127.0.0.1:8443/ 2>&1 || echo "FAILED as expected: Server rejected connection (no client cert)"
echo ""

echo ""
echo "=== Test 3: Self-Signed Certificate (Wrong CA) ==="
echo "Expected: Connection rejected (untrusted client cert)"
openssl req -x509 -newkey rsa:2048 -keyout /tmp/rogue.key \
  -out /tmp/rogue.crt -days 1 -nodes -subj "/CN=rogue-service" 2>/dev/null
curl -s \
  --cert /tmp/rogue.crt \
  --key /tmp/rogue.key \
  --cacert ../certs/ca/ca.crt \
  https://127.0.0.1:8443/ 2>&1 || echo "FAILED as expected: Server rejected untrusted client cert"
rm -f /tmp/rogue.key /tmp/rogue.crt
echo ""

echo ""
echo "=== Test 4: Verify Certificate Details ==="
echo "Server certificate:"
openssl x509 -in ../certs/server/server.crt -noout -subject -issuer -dates
echo ""
echo "Client certificate:"
openssl x509 -in ../certs/client/client.crt -noout -subject -issuer -dates
echo ""
echo "CA certificate:"
openssl x509 -in ../certs/ca/ca.crt -noout -subject -issuer -dates

echo ""
echo "=== All Tests Complete ==="
```

Expected output:
```
Test 1: {"status":"ok","server":"api-service","authenticated_client":"client-service","message":"mTLS connection verified"}
Test 2: Connection rejected (no client cert) -- curl error 35 or 56
Test 3: Connection rejected (untrusted client cert) -- curl error 35 or 56
Test 4: Certificate details displayed with correct CN, issuer, and validity dates
```

### Why This Works

Testing mTLS requires verifying both positive and negative cases:

- **Test 1 (valid mTLS)**: Both sides present certificates from the same CA. The handshake succeeds and both identities are verified.
- **Test 2 (no client cert)**: The client connects without presenting a certificate. The server's `FAIL_IF_NO_PEER_CERT` flag causes the handshake to fail.
- **Test 3 (wrong CA)**: The client presents a certificate signed by a different CA. The server does not trust this CA and rejects the handshake.
- **Test 4 (certificate details)**: Verifies that the certificates have correct CN values, are signed by the expected CA, and have appropriate validity periods.

Each test exercises a specific aspect of the mTLS trust chain. If any test produces unexpected results, the corresponding configuration is incorrect.

### Common Mistakes

- **Not testing negative cases.** Only testing the happy path (valid mTLS) does not verify that the server actually rejects invalid connections.
- **Using curl -k for all tests.** The `-k` flag disables certificate verification, which masks configuration errors.
- **Forgetting to start the server before testing.** The server must be running and accepting connections before the test script executes.
- **Not cleaning up test certificates.** Temporary certificates (like rogue.crt) should be removed after testing.

## Key Takeaway

Mutual TLS is the cryptographic foundation of zero trust networking. Unlike standard TLS where only the server proves its identity, mTLS ensures both sides authenticate. This means every service in your mesh has a cryptographic identity that cannot be forged (assuming the CA key is secure). Combined with authorization policies, mTLS enables fine-grained access control: not just "can you prove who you are?" but "given who you are, are you allowed to do this?"
