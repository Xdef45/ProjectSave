FROM debian:13.3 AS chef
RUN apt update && apt upgrade -y
RUN apt install curl pkg-config libssl-dev make build-essential -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -O && sh curl_response --quiet -y
ENV PATH=/root/.cargo/bin:$PATH
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build

# We do not need the Rust toolchain to run the binary!
FROM debian:13.3 AS runtime
COPY --from=builder /app/target/debug/strongholer /usr/local/bin
ENTRYPOINT ["/usr/local/bin/strongholer"]