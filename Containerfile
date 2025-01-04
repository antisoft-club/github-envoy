FROM docker.io/library/rust:latest as builder

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src src

RUN cargo fetch

RUN cargo build --release



FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /app/target/release/github-envoy .

EXPOSE 8081

CMD ["./github-envoy"]
