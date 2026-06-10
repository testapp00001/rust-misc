#!/bin/bash
# Generate the full directory structure for rust-security-lab
set -e

BASE="/home/khing/projects/learn-rust-DSA/rust-security-lab"

# Module definitions: "directory_name|display_name|dependencies"
MODULES=(
    "01-crypto-primitives|Cryptographic Foundations|ring,sha2,sha3,blake3,hmac,digest,base64,hex"
    "02-symmetric-encryption|Symmetric Encryption|ring,aes-gcm,chacha20poly1305,rand"
    "03-asymmetric-encryption|Asymmetric Encryption|ring,rsa,x25519-dalek,p256,rand"
    "04-digital-signatures|Digital Signatures|ed25519-dalek,p256,rand,sha2"
    "05-zero-knowledge-proofs|Zero-Knowledge Proofs|sha2,rand,curve25519-dalek"
    "06-key-management|Key Management|ring,hkdf,hmac,sha2,rand,zeroize,secrecy"
    "07-password-security|Password Security|argon2,bcrypt,scrypt,rand,zeroize,secrecy"
    "08-secure-local-storage|Secure Local Storage|ring,aes-gcm,rand,zeroize,secrecy,serde,serde_json"
    "09-database-security|Database Security|ring,aes-gcm,sha2,serde,serde_json,rand"
    "10-data-in-transit|Data in Transit|ring,sha2,rustls,rand"
    "11-authentication|Authentication|ring,sha2,hmac,serde,serde_json,rand,base64"
    "12-authorization|Authorization|serde,serde_json,ring,sha2"
    "13-secret-management|Secret Management|ring,aes-gcm,rand,zeroize,secrecy,serde,serde_json"
    "14-api-security|API Security|ring,hmac,sha2,serde,serde_json,rand"
    "15-memory-security|Memory Security|zeroize,secrecy,ring,rand"
    "16-serialization-security|Serialization Security|serde,serde_json,ring,sha2"
    "17-input-validation|Input Validation|regex,ring,sha2,url"
    "18-logging-and-audit|Logging and Audit|ring,sha2,serde,serde_json,chrono"
    "19-error-security|Error Security|thiserror,anyhow"
    "20-supply-chain-security|Supply Chain Security|ring,sha2,serde,serde_json"
    "21-fuzzing-and-testing|Fuzzing and Testing|ring,sha2,proptest"
    "22-static-analysis|Static Analysis|ring,sha2,zeroize"
    "23-threat-modeling|Threat Modeling|serde,serde_json"
    "24-secure-deployment|Secure Deployment|ring,sha2,serde,serde_json"
    "25-post-quantum-crypto|Post-Quantum Cryptography|ring,sha2,rand"
    "26-privacy-engineering|Privacy Engineering|ring,sha2,rand,serde,serde_json"
    "27-advanced-crypto-protocols|Advanced Crypto Protocols|ring,sha2,rand,serde,serde_json"
    "28-hardware-security|Hardware Security|ring,sha2,rand,serde,serde_json"
    "29-capstone-secure-messenger|Capstone: Secure Messenger|ring,aes-gcm,x25519-dalek,ed25519-dalek,sha2,hkdf,rand,zeroize,secrecy,serde,serde_json,base64"
    "30-capstone-encrypted-vault|Capstone: Encrypted Vault|ring,aes-gcm,argon2,sha2,hkdf,rand,zeroize,secrecy,serde,serde_json,base64,chrono"
    "31-capstone-secure-web-api|Capstone: Secure Web API|ring,aes-gcm,ed25519-dalek,sha2,hmac,hkdf,rand,zeroize,secrecy,serde,serde_json,base64,chrono"
)

for entry in "${MODULES[@]}"; do
    IFS='|' read -r dir_name display_name deps <<< "$entry"

    # Create directory structure
    mkdir -p "$BASE/$dir_name/src/solution"

    # Create Cargo.toml
    dep_lines=""
    for dep in $(echo "$deps" | tr ',' ' '); do
        dep_lines="$dep_lines$dep = { workspace = true }"
        echo "" > /dev/null  # noop
    done

    cat > "$BASE/$dir_name/Cargo.toml" << CARGO_EOF
[package]
name = "$dir_name"
version = "0.1.0"
edition = "2021"
description = "rust-security-lab: $display_name"

[features]
default = []
solution = []

[dependencies]
CARGO_EOF

    for dep in $(echo "$deps" | tr ',' ' '); do
        echo "$dep = { workspace = true }" >> "$BASE/$dir_name/Cargo.toml"
    done

    echo "" >> "$BASE/$dir_name/Cargo.toml"
    echo "[dev-dependencies]" >> "$BASE/$dir_name/Cargo.toml"
    echo 'criterion = { workspace = true }' >> "$BASE/$dir_name/Cargo.toml"

    echo "Created: $dir_name"
done

echo ""
echo "Structure generation complete!"
echo "Total modules: ${#MODULES[@]}"
