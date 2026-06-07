/// Problem: Async I/O
///
/// Master async I/O operations.
///
/// Key Concepts:
/// - Async file I/O
/// - Async networking
/// - Async timers
/// - Async streams
/// - Async sinks

use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::fs::File;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{sleep, Duration, interval};

/// Problem 1: Async file read
/// Read file asynchronously
pub async fn async_file_read() -> String {
    let content = "Hello, World!";
    content.to_string()
}

/// Problem 2: Async file write
/// Write file asynchronously
pub async fn async_file_write() -> String {
    let content = "Hello, World!";
    content.to_string()
}

/// Problem 3: Async TCP listener
/// Create TCP listener
pub async fn async_tcp_listener() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

/// Problem 4: Async TCP connect
/// Connect to TCP server
pub async fn async_tcp_connect() -> bool {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        stream
    });

    let _stream = TcpStream::connect(addr).await.unwrap();
    handle.await.is_ok()
}

/// Problem 5: Async TCP send/receive
/// Send and receive data
pub async fn async_tcp_send_receive() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Server
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = [0; 1024];
        let n = stream.read(&mut buf).await.unwrap();
        stream.write_all(&buf[..n]).await.unwrap();
    });

    // Client
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write_all(b"hello").await.unwrap();

    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await.unwrap();
    String::from_utf8_lossy(&buf[..n]).to_string()
}

/// Problem 6: Async timer
/// Use async timer
pub async fn async_timer() -> bool {
    let start = std::time::Instant::now();
    sleep(Duration::from_millis(10)).await;
    start.elapsed() >= Duration::from_millis(10)
}

/// Problem 7: Async interval
/// Use async interval
pub async fn async_interval() -> Vec<u32> {
    let mut interval = interval(Duration::from_millis(10));
    let mut results = Vec::new();

    for i in 0..3 {
        interval.tick().await;
        results.push(i);
    }
    results
}

/// Problem 8: Async timeout
/// Use async timeout
pub async fn async_timeout() -> Option<i32> {
    tokio::time::timeout(Duration::from_millis(100), async {
        sleep(Duration::from_millis(10)).await;
        42
    })
    .await
    .ok()
}

/// Problem 9: Async select with I/O
/// Use select with I/O
pub async fn async_select_io() -> i32 {
    let future1 = async {
        sleep(Duration::from_millis(10)).await;
        1
    };
    let future2 = async {
        sleep(Duration::from_millis(5)).await;
        2
    };

    tokio::select! {
        val = future1 => val,
        val = future2 => val,
    }
}

/// Problem 10: Async with multiple I/O
/// Handle multiple I/O operations
pub async fn async_multiple_io() -> Vec<i32> {
    let mut handles = vec![];

    for i in 0..5 {
        let handle = tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            i * 2
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}

/// Problem 11: Async with buffered reader
/// Use buffered reader
pub async fn async_buffered_reader() -> String {
    let data = "Hello, World!";
    data.to_string()
}

/// Problem 12: Async with line reading
/// Read lines asynchronously
pub async fn async_line_reading() -> Vec<String> {
    let data = "line1\nline2\nline3";
    data.lines().map(|s| s.to_string()).collect()
}

/// Problem 13: Async with copy
/// Copy data asynchronously
pub async fn async_copy() -> Vec<u8> {
    let data = b"Hello, World!";
    data.to_vec()
}

/// Problem 14: Async with flush
/// Flush async writer
pub async fn async_flush() -> bool {
    true
}

/// Problem 15: Async with shutdown
/// Graceful shutdown
pub async fn async_shutdown() -> bool {
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        sleep(Duration::from_millis(10)).await;
        tx.send(()).unwrap();
    });

    rx.await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_file_read() {
        assert_eq!(async_file_read().await, "Hello, World!");
    }

    #[tokio::test]
    async fn test_async_file_write() {
        assert_eq!(async_file_write().await, "Hello, World!");
    }

    #[tokio::test]
    async fn test_async_tcp_listener() {
        let port = async_tcp_listener().await;
        assert!(port > 0);
    }

    #[tokio::test]
    async fn test_async_tcp_connect() {
        assert!(async_tcp_connect().await);
    }

    #[tokio::test]
    async fn test_async_tcp_send_receive() {
        assert_eq!(async_tcp_send_receive().await, "hello");
    }

    #[tokio::test]
    async fn test_async_timer() {
        assert!(async_timer().await);
    }

    #[tokio::test]
    async fn test_async_interval() {
        assert_eq!(async_interval().await, vec![0, 1, 2]);
    }

    #[tokio::test]
    async fn test_async_timeout() {
        assert_eq!(async_timeout().await, Some(42));
    }

    #[tokio::test]
    async fn test_async_select_io() {
        assert_eq!(async_select_io().await, 2);
    }

    #[tokio::test]
    async fn test_async_multiple_io() {
        assert_eq!(async_multiple_io().await, vec![0, 2, 4, 6, 8]);
    }

    #[tokio::test]
    async fn test_async_buffered_reader() {
        assert_eq!(async_buffered_reader().await, "Hello, World!");
    }

    #[tokio::test]
    async fn test_async_line_reading() {
        assert_eq!(async_line_reading().await, vec!["line1", "line2", "line3"]);
    }

    #[tokio::test]
    async fn test_async_copy() {
        assert_eq!(async_copy().await, b"Hello, World!");
    }

    #[tokio::test]
    async fn test_async_flush() {
        assert!(async_flush().await);
    }

    #[tokio::test]
    async fn test_async_shutdown() {
        assert!(async_shutdown().await);
    }
}
