# ============================================================================
# Rust Security Lab — The Final Path of Rust Security Coding
# ============================================================================
# Quick reference:
#   make list                                        — List all modules
#   make test TOPIC=01-crypto-primitives             — Test a module (your solution)
#   make test TOPIC=01-crypto-primitives PROB=p01    — Test one lesson
#   make solution TOPIC=01-crypto-primitives         — Test reference solution
#   make bench TOPIC=01-crypto-primitives            — Run benchmarks
#   make test-all                                    — Run all tests (your solutions)
#   make solution-all                                — Run all tests (reference solutions)
# ============================================================================

# ── List all modules ────────────────────────────────────────────────────────
.PHONY: list
list:
	@echo "rust-security-lab — The Final Path of Rust Security Coding"
	@echo ""
	@echo "=== Section 1: Cryptographic Foundations ==="
	@echo "  01-crypto-primitives        Hashing, HMAC, encoding, checksums"
	@echo "  02-symmetric-encryption     AES-GCM, ChaCha20, authenticated encryption"
	@echo "  03-asymmetric-encryption    RSA, X25519, ECDH, hybrid encryption"
	@echo "  04-digital-signatures       Ed25519, ECDSA, batch verification"
	@echo "  05-zero-knowledge-proofs    ZK fundamentals, Schnorr, Bulletproofs"
	@echo ""
	@echo "=== Section 2: Data Protection ==="
	@echo "  06-key-management           Key generation, HKDF, rotation, hierarchy"
	@echo "  07-password-security        Argon2id, bcrypt, salting, timing attacks"
	@echo "  08-secure-local-storage     File encryption, secure deletion, temp files"
	@echo "  09-database-security        SQL injection, field encryption, audit logging"
	@echo "  10-data-in-transit          TLS 1.3, cert pinning, mTLS, replay prevention"
	@echo ""
	@echo "=== Section 3: Identity & Secrets ==="
	@echo "  11-authentication           JWT, OAuth2, TOTP/MFA, passkeys"
	@echo "  12-authorization            RBAC, ABAC, capability tokens"
	@echo "  13-secret-management        Vault, env vars, rotation, secret scanning"
	@echo "  14-api-security             Rate limiting, HMAC webhooks, CORS, CSRF"
	@echo ""
	@echo "=== Section 4: Application Hardening ==="
	@echo "  15-memory-security          Zeroize, secrecy, constant-time, side-channels"
	@echo "  16-serialization-security   Deserialization attacks, serde safety"
	@echo "  17-input-validation         Injection, XSS, SSRF, path traversal"
	@echo "  18-logging-and-audit        PII redaction, tamper-evident logs"
	@echo "  19-error-security           Information leakage, timing oracles"
	@echo "  20-supply-chain-security    cargo-audit, SBOM, reproducible builds"
	@echo ""
	@echo "=== Section 5: Security Engineering ==="
	@echo "  21-fuzzing-and-testing      cargo-fuzz, proptest, differential fuzzing"
	@echo "  22-static-analysis          Clippy, Miri, Kani, cargo-geiger"
	@echo "  23-threat-modeling          STRIDE, attack trees, DREAD"
	@echo "  24-secure-deployment        Reproducible builds, container security, SLSA"
	@echo ""
	@echo "=== Section 6: Advanced & Quantum ==="
	@echo "  25-post-quantum-crypto      Kyber, SPHINCS+, hybrid PQC"
	@echo "  26-privacy-engineering      Differential privacy, GDPR, anonymization"
	@echo "  27-advanced-crypto-protocols MPC, secret sharing, homomorphic encryption"
	@echo "  28-hardware-security        TPM, HSM, secure enclaves, attestation"
	@echo ""
	@echo "=== Section 7: Capstone Projects ==="
	@echo "  29-capstone-secure-messenger  E2E encrypted messenger"
	@echo "  30-capstone-encrypted-vault   Encrypted password vault"
	@echo "  31-capstone-secure-web-api    Production secure web API"
	@echo ""
	@echo "Usage: make test TOPIC=01-crypto-primitives"
	@echo "       make solution TOPIC=02-symmetric-encryption"

# ── Test your solution ──────────────────────────────────────────────────────
.PHONY: test
test:
ifndef TOPIC
	@echo "Usage: make test TOPIC=01-crypto-primitives [PROB=p01_hashing_basics]"
	@exit 1
endif
ifdef PROB
	cd $(TOPIC) && cargo test $(PROB) -- --nocapture
else
	cd $(TOPIC) && cargo test -- --nocapture
endif

# ── Test reference solution ─────────────────────────────────────────────────
.PHONY: solution
solution:
ifndef TOPIC
	@echo "Usage: make solution TOPIC=01-crypto-primitives [PROB=p01_hashing_basics]"
	@exit 1
endif
ifdef PROB
	cd $(TOPIC) && cargo test --features solution $(PROB) -- --nocapture
else
	cd $(TOPIC) && cargo test --features solution -- --nocapture
endif

# ── Run benchmarks ──────────────────────────────────────────────────────────
.PHONY: bench
bench:
ifndef TOPIC
	@echo "Usage: make bench TOPIC=01-crypto-primitives"
	@exit 1
endif
ifdef PROB
	cd $(TOPIC) && cargo test --features solution $(PROB) -- --ignored --nocapture
else
	cd $(TOPIC) && cargo test --features solution -- --ignored --nocapture
endif

# ── Run ALL tests across all modules ────────────────────────────────────────
.PHONY: test-all
test-all:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Running ALL tests (your solutions)"
	@echo "═══════════════════════════════════════════════════════════════"
	cargo test --workspace

# ── Run ALL reference solution tests ────────────────────────────────────────
.PHONY: solution-all
solution-all:
	@echo "═══════════════════════════════════════════════════════════════"
	@echo "  Running ALL tests (reference solutions)"
	@echo "═══════════════════════════════════════════════════════════════"
	@for dir in [0-9]*/; do \
		topic=$$(basename "$$dir"); \
		echo ""; \
		echo "▸ $$topic"; \
		echo "───────────────────────────────────────────────"; \
		cd "$$dir" && cargo test --features solution -- --nocapture 2>&1; \
		cd ..; \
	done

# ── List lessons in a module ────────────────────────────────────────────────
.PHONY: lessons
lessons:
ifndef TOPIC
	@echo "Usage: make lessons TOPIC=01-crypto-primitives"
	@exit 1
endif
	@echo "Lessons in $(TOPIC):"
	@echo ""
	@ls -1 $(TOPIC)/src/p*.rs 2>/dev/null | xargs -I{} basename {} .rs | sort

# ── Show module README ──────────────────────────────────────────────────────
.PHONY: readme
readme:
ifndef TOPIC
	@echo "Usage: make readme TOPIC=01-crypto-primitives"
	@exit 1
endif
	@cat $(TOPIC)/README.md

# ── Help ────────────────────────────────────────────────────────────────────
.PHONY: help
help:
	@echo "Rust Security Lab — The Final Path of Rust Security Coding"
	@echo ""
	@echo "Commands:"
	@echo "  make list                                  List all modules"
	@echo "  make test     TOPIC=<module> [PROB=<prob>]  Test your solution"
	@echo "  make solution TOPIC=<module> [PROB=<prob>]  Test reference solution"
	@echo "  make bench    TOPIC=<module>                Run benchmarks"
	@echo "  make lessons  TOPIC=<module>                List lessons in module"
	@echo "  make readme   TOPIC=<module>                Show module README"
	@echo "  make test-all                              Test all (your solutions)"
	@echo "  make solution-all                          Test all (reference)"
	@echo ""
	@echo "Examples:"
	@echo "  make test TOPIC=01-crypto-primitives"
	@echo "  make solution TOPIC=01-crypto-primitives PROB=p01_hashing_basics"
	@echo "  make test TOPIC=02-symmetric-encryption"
