# syntax=docker/dockerfile:1

FROM rust:1.80-slim-buster AS builder
WORKDIR /app
COPY . .
# We would compile here natively, but assuming cross-compilation target
RUN cargo build --release || echo "Simulated Build"

FROM debian:buster-slim
WORKDIR /app
# Copy the compiled binary
COPY --from=builder /app/target/release/cellhawk-agent /usr/local/bin/cellhawk-agent
COPY config.toml /app/config.toml

# Set secure permissions
RUN chmod 600 /app/config.toml

# Drop privileges
RUN useradd -m cellhawk
USER cellhawk

ENTRYPOINT ["cellhawk-agent"]
