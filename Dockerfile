FROM rust:1.84 AS builder

WORKDIR /app
RUN apt update && apt install -y lld clang 
COPY . .
ENV APP_ENVIRONMENT=production
ENV SQLX_OFFLINE=true
RUN cargo build --release
ENTRYPOINT ["./target/release/newsletter"]

FROM debian:stable-slim AS runtime

WORKDIR /app
RUN apt update -y \
  && apt install -y --no-install-recommends openssl ca-certificates \
  && apt autoremove -y \
  && apt clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/newsletter newsletter
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./newsletter"]
