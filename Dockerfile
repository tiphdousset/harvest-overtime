# Build Stage
FROM rust:1.76.0-slim-bookworm as builder
WORKDIR /usr/src/overtime

# Install runtime library
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/

# Copy the source code into the container
COPY src/ src/
COPY Cargo.toml .
COPY Cargo.lock .
COPY index.html .

# Build your application
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /root/

# Copy the build artifact from the build stage
COPY --from=builder /usr/src/overtime/target/release/overtime .

# Install runtime library
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/

# Command to run the executable
CMD ["./overtime"]
