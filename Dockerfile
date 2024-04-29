
FROM rust:latest

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src

RUN apt-get update -y
RUN apt-get upgrade -y
RUN apt-get install pkg-config libssl-dev ca-certificates -y

RUN cargo install --path .

ENTRYPOINT ["autoguard"]
