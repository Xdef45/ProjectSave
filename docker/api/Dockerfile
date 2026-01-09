FROM debian:13.2
RUN apt update && apt upgrade -y
RUN apt install git curl pkg-config libssl-dev make -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -O && sh curl_response --quiet -y
RUN apt install cargo -y
RUN useradd api -d /home/api -p "" 
WORKDIR /home/api
RUN git clone https://github.com/Xdef45/ProjectSave.git -b feature/ajoutAPI
COPY .env.json .
WORKDIR /home/api/ProjectSave/backend/strongholer
RUN cargo build
COPY .env.json .
EXPOSE 80
CMD ["cargo", "run"]
