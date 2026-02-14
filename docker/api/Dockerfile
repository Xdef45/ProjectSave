FROM rust:alpine3.23 AS chef
RUN apk update && apk upgrade
RUN apk add curl pkgconf openssl-dev make build-base libc6-compat perl
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
FROM alpine:3.14 AS runtime
RUN apk update && apk upgrade
RUN apk add openssh-client libc6-compat
COPY entry-point.sh /
COPY --from=builder /app/target/debug/strongholer /usr/local/bin/strongholer
RUN chmod u+x /entry-point.sh
ENTRYPOINT ["/entry-point.sh"]