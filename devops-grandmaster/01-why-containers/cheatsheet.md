# Cheatsheet: Why Containers

## Container vs VM

| | VM | Container |
|---|---|---|
| Size | 2-4 GB | 10-100 MB |
| Boot time | 30-60 seconds | 100ms-2 seconds |
| Isolation | Full OS-level | Process-level |
| Overhead | High (full OS) | Low (shared kernel) |
| Density | 10-20 per host | 100-1000 per host |

## Key Linux Kernel Features

```
Namespaces  → Isolation (what you can see)
Cgroups     → Resource limits (what you can use)
Union FS    → Layered images (shared layers)
```

## Container Lifecycle

```
Image → docker run → Container → docker stop → Stopped Container
                                                         ↓
                                               docker start → Container
                                               docker rm → Deleted
```

## Key Terms

| Term | Definition |
|------|-----------|
| Container | A running instance of an image |
| Image | A read-only template for creating containers |
| Dockerfile | Instructions to build an image |
| Container Runtime | Software that runs containers (Docker, containerd) |
| Registry | Storage for images (Docker Hub, ECR, GCR) |
