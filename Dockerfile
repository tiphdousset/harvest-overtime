# Build Stage
FROM rust:1.76.0-slim-bookworm as chef
RUN cargo install cargo-chef
WORKDIR /app
# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/


# Only prepares the build plan
FROM chef as planner
COPY src/ src/
COPY Cargo.toml Cargo.lock ./
# Prepare a build plan ("recipe")
RUN cargo chef prepare --recipe-path recipe.json


FROM chef as builder
# Copy the build plan from the previous Docker stage
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this layer is cached as long as `recipe.json` doesn't change.
RUN cargo chef cook --release --recipe-path recipe.json
# Copy the source code into the container
COPY src/ src/
COPY Cargo.toml Cargo.lock index.html ./
# Build your application
RUN cargo build --release


# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app
# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/
# Copy the build artifact from the build stage
COPY --from=builder /app/target/release/overtime .
# Command to run the executable
CMD ["sh", "-c", "CLICOLOR_FORCE=true ./overtime --serve"]
