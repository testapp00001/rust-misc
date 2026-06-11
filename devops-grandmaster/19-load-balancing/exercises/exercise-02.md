# Exercise 02: Nginx as a Load Balancer (Guided)

## Objective

Configure Nginx to act as a reverse-proxy load balancer for three backend HTTP
services using Docker Compose.

## Prerequisites

- Docker and Docker Compose installed
- A text editor

## Instructions

### Step 1 -- Create the project directory

```
mkdir -p ~/lb-exercise-02 && cd ~/lb-exercise-02
```

### Step 2 -- Create backend services

Create a file called `docker-compose.yml` that defines:

- **Three identical HTTP server containers** named `backend-1`, `backend-2`,
  and `backend-3`. Use the image `hashicorp/http-echo:0.2.3` (a tiny Go
  server). Each container should respond with a message that identifies itself.
  Pass the flag `-text="Hello from backend-N"` where N is 1, 2, or 3.
- **An Nginx container** named `load-balancer` that listens on port `8080` of
  the host and forwards traffic to the three backends.

Expose each backend on an internal port (e.g., `5678` -- the default for
`http-echo`).

### Step 3 -- Write the Nginx configuration

Create a file called `nginx.conf` with the following requirements:

1. Define an `upstream` block called `backend_pool` listing the three
   backends.
2. The `server` block should listen on port `80` and proxy all requests (`/`)
   to `backend_pool`.
3. Use **Round Robin** (the default) as the load-balancing algorithm.

### Step 4 -- Update docker-compose.yml

Make sure the Nginx container:

- Mounts your `nginx.conf` into the container at
  `/etc/nginx/nginx.conf`.
- Depends on the three backend containers.
- Maps host port `8080` to container port `80`.

### Step 5 -- Verify

Run:

```bash
docker compose up -d
curl http://localhost:8080
```

Repeat the `curl` command several times. You should see the responses rotate
through the three backends.

### Step 6 -- Experiment

Change the algorithm to **Least Connections** by adding `least_conn;` inside
the `upstream` block. Reload Nginx with:

```bash
docker compose exec load-balancer nginx -s reload
```

Verify that the behaviour is still correct.

## Success Criteria

- [ ] `docker compose up -d` starts all four containers without errors.
- [ ] `curl http://localhost:8080` returns a response from one of the backends.
- [ ] Repeated `curl` calls show responses from all three backends (Round Robin).
- [ ] After switching to `least_conn`, the load balancer still works.
- [ ] `nginx.conf` uses an `upstream` block (not three separate `proxy_pass`
      directives).

## Hints

<details>
<summary>Hint 1 -- upstream block syntax</summary>

```nginx
upstream backend_pool {
    server backend-1:5678;
    server backend-2:5678;
    server backend-3:5678;
}
```

Use the Docker Compose service names as hostnames. Docker's internal DNS
resolves them.

</details>

<details>
<summary>Hint 2 -- proxy_pass</summary>

Inside the `server` block:

```nginx
location / {
    proxy_pass http://backend_pool;
}
```

</details>

<details>
<summary>Hint 3 -- Mounting nginx.conf</summary>

In `docker-compose.yml`, under the `load-balancer` service:

```yaml
volumes:
  - ./nginx.conf:/etc/nginx/nginx.conf:ro
```

</details>

<details>
<summary>Hint 4 -- docker-compose.yml structure</summary>

You need four services total. The `hashicorp/http-echo` image takes flags
via `command`. Example for one backend:

```yaml
backend-1:
  image: hashicorp/http-echo:0.2.3
  command: ["-text=Hello from backend-1", "-listen=:5678"]
```

</details>
