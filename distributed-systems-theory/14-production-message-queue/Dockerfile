FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/production-message-queue /usr/local/bin/mq
COPY config/default.toml /etc/mq/config.toml
EXPOSE 9092
CMD ["mq", "--config", "/etc/mq/config.toml"]
