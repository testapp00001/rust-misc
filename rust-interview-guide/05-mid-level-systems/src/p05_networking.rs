/// Problem: Networking
///
/// Master Rust's networking capabilities.
///
/// Key Concepts:
/// - TCP/UDP
/// - HTTP
/// - Sockets
/// - DNS
/// - TLS

use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr};
use std::io::{self, Read, Write};

/// Problem 1: TCP listener
/// Create TCP listener
pub fn tcp_listener(addr: &str) -> io::Result<TcpListener> {
    TcpListener::bind(addr)
}

/// Problem 2: TCP connect
/// Connect to TCP server
pub fn tcp_connect(addr: &str) -> io::Result<TcpStream> {
    TcpStream::connect(addr)
}

/// Problem 3: TCP send/receive
/// Send and receive data
pub fn tcp_send_receive(stream: &mut TcpStream, data: &[u8]) -> io::Result<Vec<u8>> {
    stream.write_all(data)?;
    stream.flush()?;

    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer)?;
    buffer.truncate(n);
    Ok(buffer)
}

/// Problem 4: UDP socket
/// Create UDP socket
pub fn udp_socket(addr: &str) -> io::Result<UdpSocket> {
    UdpSocket::bind(addr)
}

/// Problem 5: UDP send/receive
/// Send and receive UDP data
pub fn udp_send_receive(socket: &UdpSocket, addr: &str, data: &[u8]) -> io::Result<Vec<u8>> {
    socket.send_to(data, addr)?;

    let mut buffer = vec![0; 1024];
    let (n, _) = socket.recv_from(&mut buffer)?;
    buffer.truncate(n);
    Ok(buffer)
}

/// Problem 6: DNS lookup
/// Resolve hostname
pub fn dns_lookup(hostname: &str) -> io::Result<Vec<SocketAddr>> {
    use std::net::ToSocketAddrs;
    let addrs: Vec<SocketAddr> = format!("{}:80", hostname).to_socket_addrs()?.collect();
    Ok(addrs)
}

/// Problem 7: HTTP request (simulated)
/// Simulate HTTP request
pub fn http_get(url: &str) -> String {
    format!("GET {} HTTP/1.1\r\nHost: localhost\r\n\r\n", url)
}

/// Problem 8: HTTP response (simulated)
/// Simulate HTTP response
pub fn http_response(status: u16, body: &str) -> String {
    format!(
        "HTTP/1.1 {} OK\r\nContent-Length: {}\r\n\r\n{}",
        status,
        body.len(),
        body
    )
}

/// Problem 9: Parse HTTP request
/// Parse HTTP request
pub fn parse_http_request(request: &str) -> Option<(&str, &str)> {
    let lines: Vec<&str> = request.lines().collect();
    if let Some(first_line) = lines.first() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return Some((parts[0], parts[1]));
        }
    }
    None
}

/// Problem 10: Parse HTTP response
/// Parse HTTP response
pub fn parse_http_response(response: &str) -> Option<(u16, &str)> {
    let lines: Vec<&str> = response.lines().collect();
    if let Some(first_line) = lines.first() {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(status) = parts[1].parse::<u16>() {
                let body = response.split("\r\n\r\n").nth(1).unwrap_or("");
                return Some((status, body));
            }
        }
    }
    None
}

/// Problem 11: URL parsing
/// Parse URL
pub fn parse_url(url: &str) -> Option<(&str, &str, &str)> {
    if let Some(rest) = url.strip_prefix("http://") {
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        let host = parts[0];
        let path = if parts.len() > 1 { parts[1] } else { "" };
        return Some(("http", host, path));
    }
    if let Some(rest) = url.strip_prefix("https://") {
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        let host = parts[0];
        let path = if parts.len() > 1 { parts[1] } else { "" };
        return Some(("https", host, path));
    }
    None
}

/// Problem 12: Socket address parsing
/// Parse socket address
pub fn parse_socket_addr(addr: &str) -> Option<SocketAddr> {
    addr.parse().ok()
}

/// Problem 13: TCP server (simulated)
/// Simulate TCP server
pub fn tcp_server_simulated(request: &str) -> String {
    if request.starts_with("GET") {
        "HTTP/1.1 200 OK\r\n\r\nHello, World!".to_string()
    } else {
        "HTTP/1.1 405 Method Not Allowed\r\n\r\n".to_string()
    }
}

/// Problem 14: TCP client (simulated)
/// Simulate TCP client
pub fn tcp_client_simulated(url: &str) -> String {
    format!("GET {} HTTP/1.1\r\nHost: localhost\r\n\r\n", url)
}

/// Problem 15: Connection pooling (simulated)
/// Simulate connection pooling
pub fn connection_pool_simulated() -> Vec<String> {
    vec![
        "Connection 1".to_string(),
        "Connection 2".to_string(),
        "Connection 3".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_listener() {
        let listener = tcp_listener("127.0.0.1:0").unwrap();
        assert!(listener.local_addr().is_ok());
    }

    #[test]
    fn test_tcp_connect() {
        let listener = tcp_listener("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let stream = tcp_connect(&addr.to_string());
        assert!(stream.is_ok());
    }

    #[test]
    fn test_udp_socket() {
        let socket = udp_socket("127.0.0.1:0").unwrap();
        assert!(socket.local_addr().is_ok());
    }

    #[test]
    fn test_dns_lookup() {
        let addrs = dns_lookup("localhost");
        assert!(addrs.is_ok());
    }

    #[test]
    fn test_http_get() {
        let request = http_get("/api/data");
        assert!(request.contains("GET /api/data"));
    }

    #[test]
    fn test_http_response() {
        let response = http_response(200, "Hello");
        assert!(response.contains("200 OK"));
    }

    #[test]
    fn test_parse_http_request() {
        let request = "GET /api/data HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let (method, path) = parse_http_request(request).unwrap();
        assert_eq!(method, "GET");
        assert_eq!(path, "/api/data");
    }

    #[test]
    fn test_parse_http_response() {
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nHello";
        let (status, body) = parse_http_response(response).unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, "Hello");
    }

    #[test]
    fn test_parse_url() {
        let (protocol, host, path) = parse_url("http://example.com/path").unwrap();
        assert_eq!(protocol, "http");
        assert_eq!(host, "example.com");
        assert_eq!(path, "path");
    }

    #[test]
    fn test_parse_socket_addr() {
        let addr = parse_socket_addr("127.0.0.1:8080").unwrap();
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn test_tcp_server_simulated() {
        let response = tcp_server_simulated("GET / HTTP/1.1");
        assert!(response.contains("200 OK"));
    }

    #[test]
    fn test_tcp_client_simulated() {
        let request = tcp_client_simulated("/api/data");
        assert!(request.contains("GET /api/data"));
    }

    #[test]
    fn test_connection_pool_simulated() {
        let pool = connection_pool_simulated();
        assert_eq!(pool.len(), 3);
    }
}
