FROM rust:1.84

WORKDIR /app

RUN apt update && apt install -y lld clang 

COPY . .

RUN cargo build --release

ENTRYPOINT ["./target/release/newsletter"]

