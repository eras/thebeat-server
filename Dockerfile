FROM rust:bullseye as builder
RUN apt-get update && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-build-deps
WORKDIR /work
# Trick to optimize Docker cache usage
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p backend/src && echo 'fn main() {}' > backend/src/main.rs
COPY backend/Cargo.toml backend/Cargo.lock backend/
RUN cargo build --release
RUN rm -f backend/src/main.rs
# End of trick
COPY Cargo.toml Cargo.lock backend ./
COPY backend/src/* backend/src/
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /work/target/release/thebeat-server /usr/local/bin/thebeat-server

RUN mkdir /app
WORKDIR /app
COPY frontend-js/* /app/static/
ENV ROCKET_PROFILE=release
ENTRYPOINT ["/usr/local/bin/thebeat-server"]
