# Cheatsheet: Graceful Shutdown

## Docker Stop Sequence

```
docker stop → SIGTERM → wait (10s default) → SIGKILL
```

## Graceful Shutdown in Code

### Python (Flask/Gunicorn)
```python
import signal
import sys

def graceful_shutdown(signum, frame):
    # Stop accepting new requests
    # Wait for in-flight requests to complete
    # Close database connections
    sys.exit(0)

signal.signal(signal.SIGTERM, graceful_shutdown)
```

### Node.js (Express)
```javascript
const server = app.listen(3000);

process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully');
  server.close(() => {
    // Close database connections
    process.exit(0);
  });
  // Force close after 10s
  setTimeout(() => process.exit(1), 10000);
});
```

### Go
```go
srv := &http.Server{Addr: ":8080"}

go func() {
    sigint := make(chan os.Signal, 1)
    signal.Notify(sigint, os.Interrupt, syscall.SIGTERM)
    <-sigint

    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()
    srv.Shutdown(ctx)
}()

srv.ListenAndServe()
```

## Kubernetes Pre-stop Hook
```yaml
lifecycle:
  preStop:
    exec:
      command: ["/bin/sh", "-c", "sleep 5"]
```

## Shutdown Timeout
```bash
# Docker
docker stop -t 30 my-container

# Kubernetes
spec:
  terminationGracePeriodSeconds: 30
```
