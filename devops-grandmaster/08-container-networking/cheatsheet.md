# Cheatsheet: Container Networking

## Network Drivers

| Driver | Description | Use Case |
|--------|-------------|----------|
| bridge | Default, isolated network | Single host, containers communicate |
| host | Use host's network directly | High performance, no isolation |
| none | No networking | Isolated containers |
| overlay | Multi-host networking | Docker Swarm, cross-host |
| macvlan | Direct LAN access | Containers get own IP on LAN |

## Common Commands
```bash
docker network ls                          # List networks
docker network create my-network           # Create network
docker network inspect my-network          # Inspect network
docker network connect my-network container  # Connect container
docker network disconnect my-network container  # Disconnect
docker network rm my-network               # Remove network
docker network prune                       # Remove unused
```

## Container Communication
```bash
# Containers on same network can talk by name
docker network create app-net
docker run -d --name api --network app-net my-api
docker run -d --name db --network app-net postgres

# From api container: ping db works!
```

## Port Mapping
```bash
# -p HOST:CONTAINER
docker run -p 8080:80 nginx    # localhost:8080 → container:80
docker run -p 80:80 nginx      # localhost:80 → container:80
docker run -p 127.0.0.1:8080:80 nginx  # Only localhost
```

## Docker Compose Networking
```yaml
# Services on same compose file auto-share a network
services:
  web:
    networks:
      - frontend
  api:
    networks:
      - frontend
      - backend
  db:
    networks:
      - backend

networks:
  frontend:
  backend:
```
