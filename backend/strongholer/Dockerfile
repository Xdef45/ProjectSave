FROM debian:13.2 AS chef
RUN apt update && apt upgrade -y
RUN apt install git curl pkg-config libssl-dev make -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -O && sh curl_response --quiet -y
RUN apt install cargo -y
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN apt update && apt-get install pkg-config libssl-dev make -y
RUN cargo chef cook --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build

# We do not need the Rust toolchain to run the binary!
FROM debian:13.2 AS runtime
WORKDIR /app
COPY --from=builder /app/target/debug/strongholer /usr/local/bin
ENTRYPOINT ["/usr/local/bin/strongholer"]