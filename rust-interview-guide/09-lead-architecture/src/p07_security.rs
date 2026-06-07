/// Problem: Security
///
/// Master security in Rust.
///
/// Key Concepts:
/// - Authentication
/// - Authorization
/// - Encryption
/// - Input validation
/// - Secure coding

/// Problem 1: Authentication
/// Implement authentication
pub struct Authenticator {
    users: std::collections::HashMap<String, String>,
}

impl Authenticator {
    pub fn new() -> Self {
        Self {
            users: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, username: &str, password: &str) {
        self.users.insert(username.to_string(), password.to_string());
    }

    pub fn authenticate(&self, username: &str, password: &str) -> bool {
        self.users.get(username).map_or(false, |p| p == password)
    }
}

/// Problem 2: Authorization
/// Implement authorization
pub struct Authorizer {
    permissions: std::collections::HashMap<String, Vec<String>>,
}

impl Authorizer {
    pub fn new() -> Self {
        Self {
            permissions: std::collections::HashMap::new(),
        }
    }

    pub fn grant(&mut self, role: &str, permission: &str) {
        self.permissions
            .entry(role.to_string())
            .or_insert_with(Vec::new)
            .push(permission.to_string());
    }

    pub fn check(&self, role: &str, permission: &str) -> bool {
        self.permissions
            .get(role)
            .map_or(false, |perms| perms.contains(&permission.to_string()))
    }
}

/// Problem 3: Input validation
/// Validate input
pub struct Validator {
    rules: Vec<Box<dyn Fn(&str) -> bool>>,
}

impl Validator {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Fn(&str) -> bool>) {
        self.rules.push(rule);
    }

    pub fn validate(&self, input: &str) -> bool {
        self.rules.iter().all(|rule| rule(input))
    }
}

/// Problem 4: Sanitization
/// Sanitize input
pub fn sanitize(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Problem 5: Token generation
/// Generate secure tokens
pub struct TokenGenerator {
    secret: String,
}

impl TokenGenerator {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }

    pub fn generate(&self, data: &str) -> String {
        // Simulate token generation
        format!("{}_{}", data, self.secret)
    }

    pub fn verify(&self, token: &str, data: &str) -> bool {
        token == self.generate(data)
    }
}

/// Problem 6: Password hashing (simulated)
/// Hash passwords
pub struct PasswordHasher;

impl PasswordHasher {
    pub fn hash(password: &str) -> String {
        // Simulate hashing
        format!("hashed_{}", password)
    }

    pub fn verify(password: &str, hash: &str) -> bool {
        hash == Self::hash(password)
    }
}

/// Problem 7: Rate limiting
/// Rate limit requests
pub struct RateLimiter {
    requests: std::collections::HashMap<String, Vec<std::time::Instant>>,
    max_requests: usize,
    window: std::time::Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: std::time::Duration) -> Self {
        Self {
            requests: std::collections::HashMap::new(),
            max_requests,
            window,
        }
    }

    pub fn allow(&mut self, key: &str) -> bool {
        let now = std::time::Instant::now();
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        requests.retain(|t| now.duration_since(*t) < self.window);
        if requests.len() < self.max_requests {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Problem 8: Encryption (simulated)
/// Encrypt data
pub struct Encryptor {
    key: String,
}

impl Encryptor {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }

    pub fn encrypt(&self, data: &str) -> String {
        // Simulate encryption
        format!("encrypted_{}_{}", data, self.key)
    }

    pub fn decrypt(&self, data: &str) -> String {
        // Simulate decryption
        data.replace(&format!("encrypted_"), "")
            .replace(&format!("_{}", self.key), "")
    }
}

/// Problem 9: CSRF protection
/// Protect against CSRF
pub struct CsrfProtection {
    tokens: std::collections::HashSet<String>,
}

impl CsrfProtection {
    pub fn new() -> Self {
        Self {
            tokens: std::collections::HashSet::new(),
        }
    }

    pub fn generate_token(&mut self) -> String {
        let token = format!("csrf_{}", self.tokens.len());
        self.tokens.insert(token.clone());
        token
    }

    pub fn validate_token(&self, token: &str) -> bool {
        self.tokens.contains(token)
    }
}

/// Problem 10: SQL injection prevention
/// Prevent SQL injection
pub fn sanitize_sql(input: &str) -> String {
    input.replace('\'', "''").replace(';', "")
}

/// Problem 11: XSS prevention
/// Prevent XSS
pub fn prevent_xss(input: &str) -> String {
    sanitize(input)
}

/// Problem 12: Secure headers
/// Set secure headers
pub struct SecureHeaders;

impl SecureHeaders {
    pub fn get() -> Vec<(&'static str, &'static str)> {
        vec![
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "DENY"),
            ("X-XSS-Protection", "1; mode=block"),
            ("Strict-Transport-Security", "max-age=31536000"),
        ]
    }
}

/// Problem 13: Content Security Policy
/// Set CSP
pub struct ContentSecurityPolicy;

impl ContentSecurityPolicy {
    pub fn get() -> String {
        "default-src 'self'; script-src 'self'".to_string()
    }
}

/// Problem 14: Secure cookie
/// Set secure cookies
pub struct SecureCookie {
    name: String,
    value: String,
    http_only: bool,
    secure: bool,
}

impl SecureCookie {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: value.to_string(),
            http_only: true,
            secure: true,
        }
    }

    pub fn to_header(&self) -> String {
        format!(
            "{}={}; HttpOnly; Secure; SameSite=Strict",
            self.name, self.value
        )
    }
}

/// Problem 15: Audit logging
/// Log security events
pub struct AuditLogger {
    logs: Vec<String>,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn log(&mut self, event: &str) {
        self.logs.push(format!("[AUDIT] {}", event));
    }

    pub fn get_logs(&self) -> &[String] {
        &self.logs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authentication() {
        let mut auth = Authenticator::new();
        auth.register("user1", "pass1");
        assert!(auth.authenticate("user1", "pass1"));
        assert!(!auth.authenticate("user1", "wrong"));
    }

    #[test]
    fn test_authorization() {
        let mut auth = Authorizer::new();
        auth.grant("admin", "read");
        assert!(auth.check("admin", "read"));
        assert!(!auth.check("user", "read"));
    }

    #[test]
    fn test_validation() {
        let mut validator = Validator::new();
        validator.add_rule(Box::new(|s| !s.is_empty()));
        assert!(validator.validate("test"));
        assert!(!validator.validate(""));
    }

    #[test]
    fn test_sanitization() {
        assert_eq!(sanitize("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_token_generation() {
        let generator = TokenGenerator::new("secret");
        let token = generator.generate("data");
        assert!(generator.verify(&token, "data"));
    }

    #[test]
    fn test_password_hashing() {
        let hash = PasswordHasher::hash("password");
        assert!(PasswordHasher::verify("password", &hash));
    }

    #[test]
    fn test_rate_limiting() {
        let mut limiter = RateLimiter::new(3, std::time::Duration::from_secs(1));
        assert!(limiter.allow("user1"));
        assert!(limiter.allow("user1"));
        assert!(limiter.allow("user1"));
        assert!(!limiter.allow("user1"));
    }

    #[test]
    fn test_encryption() {
        let encryptor = Encryptor::new("key");
        let encrypted = encryptor.encrypt("data");
        let decrypted = encryptor.decrypt(&encrypted);
        assert_eq!(decrypted, "data");
    }

    #[test]
    fn test_csrf_protection() {
        let mut csrf = CsrfProtection::new();
        let token = csrf.generate_token();
        assert!(csrf.validate_token(&token));
    }

    #[test]
    fn test_sql_injection() {
        assert_eq!(sanitize_sql("'; DROP TABLE users; --"), "'' DROP TABLE users --");
    }

    #[test]
    fn test_xss_prevention() {
        let result = prevent_xss("<script>alert('xss')</script>");
        assert!(result.contains("&lt;script&gt;"));
        assert!(result.contains("&#x27;"));
    }

    #[test]
    fn test_secure_headers() {
        let headers = SecureHeaders::get();
        assert!(headers.len() > 0);
    }

    #[test]
    fn test_csp() {
        let csp = ContentSecurityPolicy::get();
        assert!(csp.contains("default-src"));
    }

    #[test]
    fn test_secure_cookie() {
        let cookie = SecureCookie::new("session", "abc123");
        let header = cookie.to_header();
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("Secure"));
    }

    #[test]
    fn test_audit_logging() {
        let mut logger = AuditLogger::new();
        logger.log("login attempt");
        assert_eq!(logger.get_logs().len(), 1);
    }
}
