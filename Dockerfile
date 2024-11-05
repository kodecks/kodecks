# Use the official Rust image as the base image for the builder stage
FROM rust:latest AS builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the actual source code
COPY .git .git
COPY Cargo.toml Cargo.lock ./
COPY kodecks kodecks
COPY kodecks-bot kodecks-bot
COPY kodecks-catalog kodecks-catalog
COPY kodecks-engine kodecks-engine
COPY kodecks-server kodecks-server

# Build the Rust project
RUN cargo build --profile distribution --bin kodecks-server

# Prepare the final image
FROM debian:bookworm-slim AS runtime

# Set the working directory
WORKDIR /app

# Copy the Rust binary from the builder stage
COPY --from=builder /usr/src/app/target/distribution/kodecks-server /usr/bin

# Set the entrypoint to run the Rust binary
ENTRYPOINT ["kodecks-server", "--host", "0.0.0.0"]
