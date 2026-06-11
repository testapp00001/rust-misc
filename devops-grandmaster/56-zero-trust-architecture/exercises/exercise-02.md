# Exercise 02: Implement mTLS Between Two Services

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Implement mutual TLS (mTLS) between a Rust client and server. Unlike standard TLS where only the server presents a certificate, mTLS requires both sides to authenticate. You will generate a mini PKI, configure both services, and verify that connections without valid client certificates are rejected.

## Tasks

### Part A: Generate the Certificate Authority and Service Certificates

Create a mini PKI with a root CA, a server certificate, and a client certificate. Use `openssl` to generate all certificates.

Write a shell script `generate-certs.sh` that:

1. Creates a root Certificate Authority (CA)
2. Generates a server certificate signed by the CA
3. Generates a client certificate signed by the CA
4. Creates a directory structure for the certificates

```
certs/
  ca/
    ca.crt
    ca.key
  server/
    server.crt
    server.key
  client/
    client.crt
    client.key
```

```bash
#!/bin/bash
# generate-certs.sh

set -euo pipefail

CERT_DIR="./certs"
mkdir -p "$CERT_DIR"/{ca,server,client}

# TODO: Generate Root CA
# openssl req -x509 -newkey rsa:4096 -keyout ... -out ... -days 365 -nodes \
#   -subj "/CN=Zero Trust CA/O=Exercise/C=US"

# TODO: Generate Server CSR and sign with CA
# openssl req -newkey rsa:4096 -keyout ... -out ... -nodes \
#   -subj "/CN=api-service/O=Exercise/C=US"
# openssl x509 -req -in ... -CA ... -CAkey ... -CAcreateserial -out ... -days 90

# TODO: Generate Client CSR and sign with CA
# openssl req -newkey rsa:4096 -keyout ... -out ... -nodes \
#   -subj "/CN=client-service/O=Exercise/C=US"
# openssl x509 -req -in ... -CA ... -CAkey ... -CAcreateserial -out ... -days 90

echo "Certificates generated in $CERT_DIR"
```

<details><summary>Hint</summary>The key difference from standard TLS is that both sides need certificates from the same CA. The server must be configured to request and verify client certificates, and the client must present its certificate during the TLS handshake.</details>

### Part B: Implement the mTLS Server

Create a Rust server that requires client certificates. The server should:
- Listen on port 8443
- Require and verify client certificates signed by the CA
- Accept HTTPS requests and respond with the client's identity

```rust
// server/src/main.rs
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Load server certificate and key
    // TODO: Load CA certificate for client verification
    // TODO: Configure TLS to require client certificates
    // TODO: Accept connections and verify client identity

    let listener = TcpListener::bind("127.0.0.1:8443")?;
    println!("mTLS server listening on 127.0.0.1:8443");

    for stream in listener.incoming() {
        // TODO: Perform TLS handshake with client cert verification
        // TODO: Read request and send response with client identity
    }

    Ok(())
}
```

Use the `native-tls` or `rustls` crate for TLS implementation. The critical configuration is setting `SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT` (for openssl) or equivalent.

<details><summary>Hint</summary>With `native-tls`, use `SslAcceptor::mozilla_intermediate` and call `builder.set_verify(SslVerifyMode::PEER | SslVerifyMode::FAIL_IF_NO_PEER_CERT)`. Load the CA cert into the trust store with `builder.set_ca_file()`. For `rustls`, configure `ClientCertVerifier::require`.</details>

### Part C: Implement the mTLS Client

Create a Rust client that presents its certificate when connecting to the server:

```rust
// client/src/main.rs
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Load client certificate and key
    // TODO: Load CA certificate for server verification
    // TODO: Configure TLS with client identity
    // TODO: Connect to server and send request

    let mut stream = TcpStream::connect("127.0.0.1:8443")?;

    // TODO: Wrap stream in TLS with client certificate

    // Send HTTP request
    stream.write_all(b"GET / HTTP/1.1\r\nHost: api-service\r\n\r\n")?;

    // Read response
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    println!("Response:\n{}", response);

    Ok(())
}
```

<details><summary>Hint</summary>The client must load both its certificate and private key, plus the CA certificate to verify the server. The TLS handshake will automatically present the client certificate when the server requests it.</details>

### Part D: Test and Verify

Write test cases to verify that mTLS is working correctly:

1. **Valid mTLS connection** -- client with correct cert connects successfully
2. **No client cert** -- connection is rejected when client has no certificate
3. **Wrong CA** -- connection is rejected when client cert is signed by a different CA
4. **Expired cert** -- connection is rejected when client cert has expired

Create a test script:

```bash
#!/bin/bash
# test-mtls.sh

echo "=== Test 1: Valid mTLS connection ==="
# Should succeed
curl --cert certs/client/client.crt --key certs/client/client.key \
     --cacert certs/ca/ca.crt https://127.0.0.1:8443/ -k

echo ""
echo "=== Test 2: No client certificate ==="
# Should fail - server requires client cert
curl --cacert certs/ca/ca.crt https://127.0.0.1:8443/ -k

echo ""
echo "=== Test 3: Self-signed client cert (wrong CA) ==="
# Generate a rogue cert and try to connect
openssl req -x509 -newkey rsa:2048 -keyout /tmp/rogue.key \
  -out /tmp/rogue.crt -days 1 -nodes -subj "/CN=rogue"
curl --cert /tmp/rogue.crt --key /tmp/rogue.key \
     --cacert certs/ca/ca.crt https://127.0.0.1:8443/ -k
```

<details><summary>Hint</summary>Tests 2, 3, and 4 should all fail with TLS handshake errors. If they succeed, your server is not properly configured to require and verify client certificates. Check that `FAIL_IF_NO_PEER_CERT` is set and the CA file is loaded correctly.</details>

## Success Criteria

- [ ] Certificates generated with proper CA hierarchy (CA -> server cert, CA -> client cert)
- [ ] Server rejects connections without client certificates
- [ ] Server rejects connections with client certificates from a different CA
- [ ] Successful mTLS connection shows client identity in response
- [ ] All four test cases pass (2 successes, 2 expected failures)

## What You Should Understand After This Exercise

- How mTLS differs from standard TLS -- both sides present and verify certificates
- The role of the Certificate Authority in establishing trust
- How to configure TLS libraries to enforce mutual authentication
- Why mTLS is a foundational building block for zero trust networking
- How certificate rotation and expiration play a role in operational security
