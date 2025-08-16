# GoldCoin Neural - Docker Container for Pool Testing
# Safe isolation for Goldcoinpool.com testing

FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Clone and build GoldCoin Neural
WORKDIR /build
RUN git clone -b goldcoin-neural https://github.com/microguy/goldcoin-neural.git
WORKDIR /build/goldcoin-neural

# Build the mining daemon
RUN cargo build --release --bin goldcoin-neurald

# Runtime stage - smaller final image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /build/goldcoin-neural/target/release/goldcoin-neurald /usr/local/bin/

# Create user for security
RUN useradd -m -u 1000 goldcoin
USER goldcoin

# Expose RPC port
EXPOSE 8122

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD goldcoin-neurald mining-status || exit 1

# Default command for pool mode
ENTRYPOINT ["goldcoin-neurald"]
CMD ["--pool-mode", "--network", "mainnet", "--rpc-port", "8122"]