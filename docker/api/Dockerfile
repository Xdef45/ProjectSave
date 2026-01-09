FROM debian:13.2
RUN apt update && apt upgrade -y
RUN apt install git curl pkg-config libssl-dev make -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -O && sh curl_response --quiet -y
RUN useradd api -d /home/api  cargo-p "" 
WORKDIR /home/api
RUN git clone https://github.com/Xdef45/ProjectSave.git -b develop
WORKDIR /home/api/ProjectSave/ProjectSave/backend/strongholer
EXPOSE 80
CMD ["cargo", "run"]
