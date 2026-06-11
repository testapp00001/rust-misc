# Exercise 04: Database Connections During Graceful Shutdown

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

Handle database connections correctly during graceful shutdown. When an
application shuts down, database connections must be drained and closed
cleanly. Open transactions must be committed or rolled back. Connection pools
must be shut down in the right order. This exercise challenges you to reason
about the sequencing of shutdown steps when a database is involved.

## Scenario

You are running a web application that uses a PostgreSQL connection pool
(10 connections). The application handles financial transactions. During a
deployment, 3 requests are in-flight:

```
Request A: Writing a transfer record (INSERT, not yet committed)
Request B: Reading account balance (SELECT, in progress)
Request C: Updating account balance (UPDATE, waiting for a lock)

Connection pool: 10 total, 3 active, 7 idle
```

A SIGTERM arrives. You must design the shutdown sequence so that:
- No transactions are left in an ambiguous state
- No data is corrupted
- The application exits within 30 seconds

---

## Tasks

### Part A: Design the Shutdown Sequence

Write the steps of the shutdown handler in order. Each step should be numbered
and include what happens and why it must happen in that order.

Consider:
1. When do you stop accepting new requests?
2. When do you stop checking out new connections from the pool?
3. When do you wait for in-flight requests?
4. When do you close idle connections?
5. When do you close active connections?
6. When do you close the connection pool itself?

<details>
<summary>Hint 1</summary>

The order matters. You cannot close the connection pool while requests are
still using it. You must first stop new work, then wait for existing work
to finish, then clean up resources.

</details>

<details>
<summary>Hint 2</summary>

Think about the difference between idle connections (sitting in the pool,
not being used) and active connections (currently executing a query or in
the middle of a transaction). You can close idle connections immediately,
but active connections must wait.

</details>

### Part B: Implement the Handler

Complete the Python code for a shutdown handler that manages database
connections correctly:

```python
import signal
import sys
import threading
from flask import Flask, jsonify
from sqlalchemy import create_engine
from sqlalchemy.orm import sessionmaker, scoped_session

app = Flask(__name__)
engine = create_engine("postgresql://user:pass@db:5432/mydb", pool_size=10)
Session = scoped_session(sessionmaker(bind=engine))

shutting_down = False
in_flight = 0
lock = threading.Lock()

@app.before_request
def before_request():
    global in_flight
    with lock:
        if shutting_down:
            return jsonify({"error": "Shutting down"}), 503
        in_flight += 1

@app.after_request
def after_request(response):
    global in_flight
    with lock:
        in_flight -= 1
    return response

@app.route('/transfer', methods=['POST'])
def transfer():
    session = Session()
    try:
        # Simulate a financial transaction
        session.execute("UPDATE accounts SET balance = balance - 100 WHERE id = 1")
        session.execute("UPDATE accounts SET balance = balance + 100 WHERE id = 2")
        session.commit()
        return jsonify({"status": "transferred"})
    except Exception:
        session.rollback()
        return jsonify({"error": "transfer failed"}), 500
    finally:
        # TODO: What should happen with the session here?
        pass

def graceful_shutdown(signum, frame):
    global shutting_down
    shutting_down = True
    print(f"[SHUTDOWN] Signal {signum} received")

    # TODO: Step 1 - Wait for in-flight requests to complete
    # TODO: Step 2 - Close the database connection pool
    # TODO: Step 3 - Exit

    pass

signal.signal(signal.SIGTERM, graceful_shutdown)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, threaded=True)
```

What goes in the `graceful_shutdown` function? What about the `finally` block
in the `/transfer` route?

<details>
<summary>Hint 1</summary>

In the `finally` block, you should call `Session.remove()` to return the
session to the pool. This ensures that even if the request fails, the
session is not left in a dirty state.

</details>

<details>
<summary>Hint 2</summary>

In `graceful_shutdown`, first wait for `in_flight` to reach 0 (with a timeout).
Then call `engine.dispose()` to close all connections in the pool. The order
is critical: if you dispose the engine first, in-flight requests will crash
when they try to use a closed connection.

</details>

### Part C: Handle the Timeout Case

What happens if in-flight requests do not complete within the grace period?
Design the behavior:

1. Should you force-rollback open transactions?
2. Should you force-close database connections?
3. What log messages should you emit?
4. What HTTP status code should the in-flight requests receive?

Write the timeout handling code.

<details>
<summary>Hint</summary>

If the timeout expires, you must force cleanup. Roll back any open transactions,
close all connections, and exit. Log the fact that you are force-shutting down
and include the count of requests that did not complete. The in-flight requests
will likely receive a connection error.

</details>

### Part D: Connection Pool Sizing During Shutdown

Your connection pool has 10 connections and 3 are active during shutdown.
Explain:

1. What happens to the 7 idle connections?
2. What happens to the 3 active connections when their requests complete?
3. Is there a scenario where you need to handle connections that are checked
   out but the request has already returned an error to the client?
4. How does `scoped_session` interact with threading during shutdown?

<details>
<summary>Hint</summary>

`scoped_session` uses thread-local storage. Each thread gets its own session.
During shutdown, the thread-local sessions need to be removed before the
underlying connections can be closed. `Session.remove()` handles this.

</details>

---

## Success Criteria

- [ ] The shutdown sequence is correctly ordered: stop requests, wait, close pool, exit
- [ ] Idle connections are closed before active connections
- [ ] Active connections are not closed while a transaction is in progress
- [ ] The timeout case handles force-rollback and force-close correctly
- [ ] The code uses `Session.remove()` to properly clean up scoped sessions
- [ ] You can explain what happens to each of the 10 connections during shutdown

## What You Should Understand After This Exercise

Database connections add a critical layer of complexity to graceful shutdown.
The shutdown sequence must respect the order: stop accepting new work, drain
in-flight work, then clean up resources. Closing connections too early causes
in-flight requests to crash. Not closing them at all leaks resources. The
timeout case is especially important in production -- you must have a plan
for when cleanup does not complete in time, because SIGKILL does not care
about your database transactions.
