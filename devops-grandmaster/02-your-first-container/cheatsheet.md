# Cheatsheet: Docker Commands

## Container Lifecycle
```bash
docker run <image>              # Create and start
docker run -it <image> bash     # Interactive with terminal
docker run -d <image>           # Detached (background)
docker run --name <n> <image>   # Named container
docker run -p 8080:80 <image>   # Port mapping
docker run --rm <image>         # Auto-remove when stopped
docker start <container>        # Start stopped container
docker stop <container>         # Stop running container
docker restart <container>      # Restart container
docker rm <container>           # Remove stopped container
docker rm -f <container>        # Force remove running
```

## Inspection
```bash
docker ps                       # Running containers
docker ps -a                    # All containers
docker logs <container>         # View logs
docker logs -f <container>      # Follow logs
docker inspect <container>      # Full details (JSON)
docker top <container>          # Processes inside
docker stats <container>        # Resource usage
```

## Execution
```bash
docker exec -it <container> bash  # Shell into container
docker exec <container> <cmd>     # Run command in container
```

## Images
```bash
docker images                   # List images
docker pull <image>             # Download image
docker rmi <image>              # Remove image
docker image prune              # Remove unused images
```

## Cleanup
```bash
docker system prune             # Remove all unused data
docker system df                # Show disk usage
```

## Common Flags

| Flag | Purpose |
|------|---------|
| `-it` | Interactive + TTY |
| `-d` | Detached (background) |
| `--name` | Container name |
| `-p HOST:CONTAINER` | Port mapping |
| `--rm` | Auto-remove on exit |
| `-v HOST:CONTAINER` | Volume mount |
| `-e KEY=VALUE` | Environment variable |
| `--network` | Network to use |
| `--restart` | Restart policy |
