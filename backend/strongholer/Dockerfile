FROM debian:13.2
RUN apt update && apt upgrade -y
RUN apt install git curl pkg-config libssl-dev make -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -O && sh curl_response --quiet -y
RUN apt install cargo -y
RUN useradd api -d /home/api -p "" 
WORKDIR /home/api
COPY Cargo.toml .
RUN mkdir src 
COPY main.rs src/
RUN cargo build
COPY ./src ./src
RUN cargo build
EXPOSE 8080
CMD ["cargo", "run"]
