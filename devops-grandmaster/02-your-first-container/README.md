# Module 02: Your First Container

## The Problem: How Do You Actually Run a Container?

You know **why** containers exist. Now you need to **use** them.

## Step 1: Install Docker

### Linux (Ubuntu/Debian)
```bash
# Update packages
sudo apt-get update

# Install Docker
sudo apt-get install -y docker.io

# Add your user to docker group (no more sudo!)
sudo usermod -aG docker $USER

# Log out and back in, then verify
docker --version
```

### macOS
```bash
# Download Docker Desktop from docker.com
# Or use Homebrew:
brew install --cask docker
```

### Windows (WSL2)
```bash
# Download Docker Desktop from docker.com
# Enable WSL2 integration in Docker Desktop settings
```

## Step 2: Run Your First Container

```bash
# Run the hello-world container
docker run hello-world
```

What happened:
1. Docker looked for the `hello-world` image locally — not found
2. Docker pulled it from Docker Hub (the default registry)
3. Docker created a container from the image
4. The container ran, printed a message, and exited

```bash
# Expected output:
# Hello from Docker!
# This message shows that your installation appears to be working correctly.
```

## Step 3: Understanding What Happened

```bash
# See all images you've downloaded
docker images

# Output:
# REPOSITORY    TAG       IMAGE ID       CREATED        SIZE
# hello-world   latest    d2c94e258dcb   6 months ago   13.3kB
```

```bash
# See all containers (including stopped ones)
docker ps -a

# Output:
# CONTAINER ID   IMAGE         COMMAND    CREATED          STATUS
# abc123         hello-world   "/hello"   5 seconds ago    Exited (0)
```

## Step 4: Run an Interactive Container

```bash
# Run Ubuntu interactively
docker run -it ubuntu bash

# Now you're INSIDE the container!
# root@abc123:/# 

# Try some commands
whoami          # root
cat /etc/os-release  # Ubuntu 22.04
ls /            # Standard Linux filesystem
exit            # Leave the container
```

**Key flags:**
- `-i` — Interactive (keeps STDIN open)
- `-t` — TTY (gives you a terminal)
- Combined `-it` = "give me a terminal inside the container"

## Step 5: Run a Background Container

```bash
# Run Nginx web server in the background
docker run -d --name my-web -p 8080:80 nginx

# What this means:
# -d           = detach (run in background)
# --name       = give it a name
# -p 8080:80   = map host port 8080 to container port 80
# nginx        = the image to use
```

```bash
# Check it's running
docker ps

# Visit http://localhost:8080 in your browser!
```

## Step 6: Container Lifecycle

```bash
# List running containers
docker ps

# Stop a container
docker stop my-web

# Start it again
docker start my-web

# Restart a container
docker restart my-web

# Remove a stopped container
docker rm my-web

# Force remove a running container
docker rm -f my-web
```

## Step 7: See What's Inside

```bash
# Run a container and look around
docker run -it --rm ubuntu bash

# --rm = automatically remove when you exit

# Inside the container:
apt-get update && apt-get install -y curl
curl -s https://api.github.com | head -20
exit  # Container is deleted automatically
```

## Step 8: Execute Commands in Running Container

```bash
# Start a container
docker run -d --name my-nginx nginx

# Execute a command inside it
docker exec my-nginx cat /etc/nginx/nginx.conf

# Open a shell inside a running container
docker exec -it my-nginx bash

# Now you're inside! Type 'exit' to leave.
```

## Step 9: See Container Logs

```bash
# View logs
docker logs my-nginx

# Follow logs in real-time (like tail -f)
docker logs -f my-nginx

# Show last 10 lines
docker logs --tail 10 my-nginx

# Show timestamps
docker logs -t my-nginx
```

## Step 10: Inspect a Container

```bash
# Full details (JSON)
docker inspect my-nginx

# Get just the IP address
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' my-nginx

# See resource usage
docker stats my-nginx

# See processes inside container
docker top my-nginx
```

## Common Commands Reference

```bash
# IMAGES
docker images              # List images
docker pull <image>        # Download image
docker rmi <image>         # Remove image
docker image prune         # Remove unused images

# CONTAINERS
docker run <image>         # Create and start
docker ps                  # List running
docker ps -a               # List all (including stopped)
docker stop <container>    # Stop
docker start <container>   # Start
docker rm <container>      # Remove
docker exec -it <container> bash  # Shell into running container
docker logs <container>    # View logs
docker inspect <container> # Full details

# CLEANUP
docker system prune        # Remove all unused data
docker system df           # Show disk usage
```

## Hands-On Exercise

### Exercise 1: Run Different Operating Systems
```bash
# Run these and explore each one:
docker run -it --rm alpine sh        # Minimal Linux (5MB!)
docker run -it --rm ubuntu bash      # Full Ubuntu
docker run -it --rm debian bash      # Debian
docker run -it --rm centos bash      # CentOS (if available)

# For each one, check:
# - cat /etc/os-release
# - ls /
# - which python3 (if installed)
# - Size of the image (docker images)
```

### Exercise 2: Run a Web Application
```bash
# Run a simple Python web server
docker run -d --name python-web -p 5000:5000 python:3.11-slim \
  python -m http.server 5000

# Visit http://localhost:5000
# Check logs: docker logs python-web
# Stop and remove: docker rm -f python-web
```

### Exercise 3: Explore Container Isolation
```bash
# Terminal 1: Run a container
docker run -it --name isolated ubuntu bash
# Create a file: echo "hello" > /tmp/test.txt

# Terminal 2: Run another container
docker run -it --name another ubuntu bash
# Try to find the file: cat /tmp/test.txt
# File doesn't exist! Containers are isolated.

# Clean up
docker rm -f isolated another
```

## Limitation: Running Existing Images is Easy, But...

You can run other people's containers. But what about **your own application**?

**Next problem:** How do you package YOUR code into a container?

→ **Next module:** [03-writing-dockerfiles](../03-writing-dockerfiles/) — Build your own container image

## Checklist

- [ ] Docker is installed and working
- [ ] I can run, stop, start, and remove containers
- [ ] I understand `-it`, `-d`, `--name`, `-p`, `--rm` flags
- [ ] I can execute commands inside a running container
- [ ] I can view container logs and inspect container details
- [ ] I understand container isolation
