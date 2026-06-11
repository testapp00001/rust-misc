# Exercise 02: Implement a Graceful Shutdown Handler

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Implement a graceful shutdown handler in a Python Flask application that catches
SIGTERM, stops accepting new requests, waits for in-flight requests to complete,
and exits cleanly. This is the core skill that makes zero-downtime deployments
possible.

## Scenario

You have a basic Flask application. Currently, when the container is stopped,
any in-flight requests are dropped. Users see "Connection reset" errors.
Your task is to add graceful shutdown handling so that in-flight requests
complete before the process exits.

---

## Tasks

### Part A: Write the Shutdown Handler

Complete the Python code below. The missing parts are marked with `# TODO`.
Your handler must:

1. Catch SIGTERM signals
2. Set a flag that stops new requests from being accepted
3. Wait for in-flight requests to finish (up to a timeout)
4. Exit cleanly

```python
import signal
import sys
import threading
from flask import Flask, jsonify

app = Flask(__name__)

# State tracking
shutting_down = False
in_flight = 0
lock = threading.Lock()

@app.before_request
def before_request():
    global in_flight
    with lock:
        # TODO: What should happen if shutting_down is True?
        # TODO: How do you track that a new request has started?
        pass

@app.after_request
def after_request(response):
    global in_flight
    with lock:
        # TODO: How do you track that a request has finished?
        pass
    return response

@app.route('/health')
def health():
    # TODO: What should the health endpoint return during shutdown?
    pass

@app.route('/slow')
def slow():
    import time
    time.sleep(5)
    return jsonify({"result": "done"})

def graceful_shutdown(signum, frame):
    global shutting_down
    # TODO: Set the shutting_down flag
    # TODO: Print a message with the current in_flight count
    # TODO: Wait for in_flight to reach 0 (with a timeout)
    # TODO: Exit cleanly
    pass

# TODO: Register the signal handler for SIGTERM

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, threaded=True)
```

<details>
<summary>Hint 1</summary>

In `before_request`, if `shutting_down` is True, return a 503 response
immediately instead of processing the request. If not shutting down,
increment `in_flight`.

</details>

<details>
<summary>Hint 2</summary>

In `after_request`, decrement `in_flight`. Use a `threading.Lock()` to make
the counter thread-safe, since Flask can handle multiple requests concurrently.

</details>

<details>
<summary>Hint 3</summary>

The graceful shutdown function should loop, sleeping 1 second per iteration,
checking if `in_flight` has reached 0 or if the timeout has expired. Register
it with `signal.signal(signal.SIGTERM, graceful_shutdown)`.

</details>

### Part B: Write the Docker Configuration

Create a `Dockerfile` and `docker-compose.yml` that give the application enough
time to shut down gracefully. The application's p99 latency is 8 seconds, and
cleanup takes about 2 seconds.

<details>
<summary>Hint</summary>

In `docker-compose.yml`, use `stop_grace_period` to set the timeout. It should
be longer than the time needed for the slowest request plus cleanup.

</details>

### Part C: Test Your Implementation

Write the sequence of bash commands you would use to verify that your graceful
shutdown works. Your test should:

1. Start the application
2. Send a slow request (use `curl` with a background process)
3. Stop the container gracefully (not kill)
4. Verify the slow request completed successfully

<details>
<summary>Hint</summary>

Use `curl http://localhost:5000/slow &` to send a background request, then
immediately run `docker compose stop app`. The request should complete
before the container stops. Check the exit status of the curl process.

</details>

### Part D: The Health Endpoint During Shutdown

Explain why the `/health` endpoint should return 503 during shutdown. What
would happen if it continued returning 200? How does this interact with a
load balancer that performs health checks?

<details>
<summary>Hint</summary>

If the health check returns 200 during shutdown, the load balancer thinks this
instance is still healthy and continues sending new requests to it. But the
application is shutting down, so those new requests will be rejected or dropped.

</details>

---

## Success Criteria

- [ ] The shutdown handler catches SIGTERM and sets a `shutting_down` flag
- [ ] New requests receive 503 when the application is shutting down
- [ ] The handler waits for in-flight requests to complete with a configurable timeout
- [ ] The health endpoint returns 503 during shutdown
- [ ] The Docker configuration provides sufficient grace period for request completion
- [ ] You can demonstrate the difference between `docker stop` (graceful) and `docker kill` (forced)

## What You Should Understand After This Exercise

A graceful shutdown handler is not optional for production applications. Without
one, every deployment causes user-facing errors. The handler must coordinate
three things: stop accepting new work, finish existing work, and report its
status to the load balancer. The Docker or Kubernetes grace period is your
deadline -- if you exceed it, SIGKILL ends everything regardless.
