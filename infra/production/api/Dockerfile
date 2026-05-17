FROM rust:1-bookworm AS builder
WORKDIR /app
RUN rustup show active-toolchain | awk '{print $1}' > /tmp/rust-toolchain-docker
COPY . .
RUN RUSTUP_TOOLCHAIN="$(cat /tmp/rust-toolchain-docker)" cargo build --release -p stellartrail-api

FROM rust:1-bookworm AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/stellartrail-api /usr/local/bin/stellartrail-api
COPY --from=builder /app/content /app/content
RUN command -v curl >/dev/null && test -f /etc/ssl/certs/ca-certificates.crt
EXPOSE 8080
CMD ["stellartrail-api"]
