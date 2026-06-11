# Solution 04: Database Connections During Graceful Shutdown

## Part A: Design the Shutdown Sequence

Here is the correct ordering of shutdown steps:

```
Step 1: Set shutting_down flag
        - New HTTP requests receive 503
        - No new database connections will be checked out
        WHY FIRST: Prevents new work from starting immediately

Step 2: Wait for in-flight requests to complete (with timeout)
        - Request A: INSERT completes, commits, session returned to pool
        - Request B: SELECT completes, session returned to pool
        - Request C: UPDATE completes (lock acquired), commits, session returned
        WHY SECOND: All database activity must finish before pool cleanup

Step 3: Close idle connections in the pool
        - 7 idle connections closed immediately
        WHY THIRD: Idle connections are safe to close -- nothing is using them

Step 4: Close the connection pool (engine.dispose())
        - By this point, all sessions have been returned to the pool
        - dispose() closes all remaining connections
        WHY FOURTH: Only close the pool after all sessions are returned

Step 5: Flush logs and metrics
        WHY FIFTH: Logging uses stdout/files, not database connections

Step 6: Exit (sys.exit(0))
        WHY LAST: Everything is cleaned up, safe to exit
```

### Why This Order Matters

The critical constraint is: **you cannot close database connections while
requests are using them.** This means:

- Steps 1 and 2 must happen before step 3 (stop new work, drain existing work)
- Step 3 (idle connections) can happen any time after step 1, but doing it
  after step 2 is simpler and avoids edge cases
- Step 4 (pool disposal) must happen after step 2 because active sessions
  return connections to the pool when requests complete

If you close the pool (step 4) before in-flight requests complete (step 2),
the requests will crash with "connection closed" errors, potentially leaving
transactions in an ambiguous state.

### Common Mistakes to Avoid

- **Closing the pool in the signal handler before waiting for requests.** This
  is the most common mistake. The handler runs in the main thread while requests
  are in other threads. If you dispose the engine immediately, the request
  threads crash.
- **Not waiting for in-flight requests at all.** Some implementations just
  set the flag and exit. This drops all in-flight requests.
- **Closing connections in the wrong order.** Closing active connections before
  idle ones can cause connection pool state corruption in some pool
  implementations.

## Part B: Implement the Handler

Here is the complete, working implementation:

```python
import signal
import sys
import time
import threading
from flask import Flask, jsonify
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, scoped_session

app = Flask(__name__)
engine = create_engine(
    "postgresql://user:pass@db:5432/mydb",
    pool_size=10,
    pool_pre_ping=True
)
Session = scoped_session(sessionmaker(bind=engine))

shutting_down = False
in_flight = 0
lock = threading.Lock()

@app.before_request
def before_request():
    global in_flight
    with lock:
        if shutting_down:
            return jsonify({"error": "Service is shutting down"}), 503
        in_flight += 1

@app.after_request
def after_request(response):
    global in_flight
    with lock:
        in_flight -= 1
    return response

@app.route('/health')
def health():
    if shutting_down:
        return jsonify({"status": "shutting_down"}), 503
    return jsonify({"status": "ok", "in_flight": in_flight}), 200

@app.route('/transfer', methods=['POST'])
def transfer():
    session = Session()
    try:
        session.execute("UPDATE accounts SET balance = balance - 100 WHERE id = 1")
        session.execute("UPDATE accounts SET balance = balance + 100 WHERE id = 2")
        session.commit()
        return jsonify({"status": "transferred"})
    except Exception:
        session.rollback()
        return jsonify({"error": "transfer failed"}), 500
    finally:
        Session.remove()

def graceful_shutdown(signum, frame):
    global shutting_down
    shutting_down = True
    print(f"[SHUTDOWN] Signal {signum} received.")

    # Phase 1: Wait for in-flight requests
    timeout = 25
    start = time.time()
    while in_flight > 0 and (time.time() - start) < timeout:
        print(f"[SHUTDOWN] Waiting for {in_flight} requests...")
        time.sleep(1)

    if in_flight > 0:
        print(f"[SHUTDOWN] Timeout! {in_flight} requests did not complete.")
    else:
        print("[SHUTDOWN] All requests completed.")

    # Phase 2: Close database connections
    print("[SHUTDOWN] Closing database connection pool...")
    Session.remove()
    engine.dispose()

    print("[SHUTDOWN] Cleanup complete. Exiting.")
    sys.exit(0)

signal.signal(signal.SIGTERM, graceful_shutdown)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, threaded=True)
```

### Why This Works

**The `finally` block: `Session.remove()`**

`Session.remove()` does two things:
1. It calls `session.close()` on the thread-local session, which rolls back
   any uncommitted transactions and returns the connection to the pool.
2. It removes the session from thread-local storage, so the next request in
   this thread gets a fresh session.

This is critical because `scoped_session` binds sessions to threads. If a
request fails partway through, the `finally` block ensures the session is
cleaned up. Without it, the connection stays checked out and the pool
eventually runs out of connections.

**The shutdown handler: `Session.remove()` then `engine.dispose()`**

The order is important:
1. `Session.remove()` -- Cleans up the scoped session registry, closing any
   remaining thread-local sessions and returning connections to the pool.
2. `engine.dispose()` -- Closes all connections in the pool (both idle and
   returned).

If you call `engine.dispose()` without `Session.remove()` first, the
scoped session registry still holds references to sessions that point to
now-closed connections. The next request that tries to use one of these
sessions would get a confusing error.

### Common Mistakes to Avoid

- **Not calling `Session.remove()` in the `finally` block.** Without it,
  connections leak on every request that uses a session. Under normal
  operation this is slow; during shutdown it means connections are never
  returned to the pool.
- **Calling `engine.dispose()` before waiting for in-flight requests.** This
  closes connections that are actively being used by request threads.
- **Not using `scoped_session`.** Without it, you need to manually manage
  session lifecycle per request, which is error-prone.

## Part C: Handle the Timeout Case

Here is the timeout handling code:

```python
def graceful_shutdown(signum, frame):
    global shutting_down
    shutting_down = True
    print(f"[SHUTDOWN] Signal {signum} received. "
          f"{in_flight} in-flight requests.")

    # Phase 1: Wait for in-flight requests (25 seconds)
    timeout = 25
    start = time.time()
    while in_flight > 0 and (time.time() - start) < timeout:
        print(f"[SHUTDOWN] Waiting for {in_flight} requests...")
        time.sleep(1)

    if in_flight > 0:
        print(f"[SHUTDOWN] TIMEOUT: {in_flight} requests did not complete "
              f"within {timeout}s. Force-closing.")
        # Force-rollback any open transactions
        # Session.remove() will rollback uncommitted transactions
    else:
        print("[SHUTDOWN] All requests completed successfully.")

    # Phase 2: Force-close all database connections
    print("[SHUTDOWN] Closing database connection pool...")
    try:
        Session.remove()
    except Exception as e:
        print(f"[SHUTDOWN] Error removing sessions: {e}")

    try:
        engine.dispose()
    except Exception as e:
        print(f"[SHUTDOWN] Error disposing engine: {e}")

    print("[SHUTDOWN] Cleanup complete. Exiting.")
    sys.exit(0)
```

### What happens during timeout:

1. **Force-rollback**: `Session.remove()` calls `session.close()` on all
   thread-local sessions. SQLAlchemy's `session.close()` rolls back any
   uncommitted transactions automatically. This is safe because the
   transactions were not completed -- rolling them back leaves the database
   in a consistent state.

2. **Force-close connections**: `engine.dispose()` closes all connections
   in the pool, regardless of state. Any request threads that try to use
   these connections will get a "connection closed" error. This is acceptable
   because the requests already timed out.

3. **Log messages**: The timeout case emits clear log messages indicating
   that force-cleanup occurred and how many requests were abandoned.

4. **HTTP status**: The in-flight requests will receive an error. The exact
   error depends on when the connection is closed:
   - If the request is between SQL statements: the next statement fails
     with "connection closed"
   - If the request is mid-transaction: the transaction is rolled back
   - The client may see a 500 error or a connection reset

### Common Mistakes to Avoid

- **Not logging the timeout case.** If you silently force-close connections,
  you will not know in production that requests were dropped.
- **Letting exceptions in cleanup prevent exit.** Wrap each cleanup step in
  try/except so that a failure in one step does not prevent the others from
  running.

## Part D: Connection Pool Sizing During Shutdown

### What happens to the 7 idle connections?

When `engine.dispose()` is called, all idle connections are closed immediately.
There is no waiting, no in-progress work. The pool releases them and they are
closed at the TCP level.

### What happens to the 3 active connections when their requests complete?

When a request completes, the `finally` block calls `Session.remove()`, which:
1. Calls `session.close()` -- rolls back any uncommitted transaction
2. Returns the connection to the pool (now it is idle)
3. Removes the session from thread-local storage

After all 3 requests complete, the 3 previously-active connections are now
idle in the pool. When `engine.dispose()` is called, they are closed along
with the other idle connections.

### Is there a scenario where connections are checked out but the request has returned?

Yes. This happens when:
- A request spawns a background thread that uses a database session
- A request uses a session in a callback or async handler that runs after
  the response is sent

In these cases, `Session.remove()` in the `after_request` hook or `finally`
block might not catch the session. The connection stays checked out until
the background work completes or the thread exits.

To handle this during shutdown, you can set a connection timeout on the
pool:

```python
engine = create_engine(
    "postgresql://user:pass@db:5432/mydb",
    pool_size=10,
    pool_timeout=5,         # Wait max 5s for a connection
    pool_recycle=3600,       # Recycle connections after 1 hour
    pool_pre_ping=True       # Test connections before use
)
```

### How does `scoped_session` interact with threading during shutdown?

`scoped_session` uses a `ThreadLocalRegistry` internally. Each thread has
its own session. During shutdown:

1. The main thread (signal handler) calls `Session.remove()` -- this removes
   the main thread's session.
2. Request threads call `Session.remove()` in their `finally` blocks -- each
   removes its own thread-local session.
3. After all threads have called `Session.remove()`, the registry is empty
   and all connections are back in the pool.

The key insight is that `Session.remove()` only affects the calling thread's
session. The signal handler cannot remove sessions from other threads. This
is why each request must have its own `Session.remove()` in a `finally` block.

### Common Mistakes to Avoid

- **Assuming `Session.remove()` in the signal handler cleans up all threads.**
  It does not. It only removes the signal handler thread's session.
- **Not using `pool_pre_ping=True`.** During shutdown, connections in the pool
  might go stale. `pool_pre_ping` tests connections before checking them out.
- **Setting pool_size too small.** If the pool is exhausted during normal
  operation, requests block waiting for connections. During shutdown, this
  can cause the drain phase to take longer than expected.

## Key Takeaway

Database connections add a critical layer of complexity to graceful shutdown.
The shutdown sequence must respect the order: stop accepting new work, drain
in-flight work, then clean up resources. The `scoped_session` pattern requires
each request to clean up its own session in a `finally` block. The signal
handler cannot clean up sessions from other threads. The timeout case requires
force-rollback and force-close, which is safe because uncommitted transactions
can be rolled back without data loss. Getting this order wrong causes
connection leaks, transaction corruption, or dropped requests.
