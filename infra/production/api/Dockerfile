FROM rust:1.95.0-bookworm AS builder
WORKDIR /app
ARG STELLARTAIL_DEPLOY_TIMEZONE=
ARG APP_COMMIT_HASH=
RUN if [ "$STELLARTAIL_DEPLOY_TIMEZONE" = "Asia/Shanghai" ]; then \
      mkdir -p /usr/local/cargo && \
      printf '%s\n' \
        '[source.crates-io]' \
        'replace-with = "rsproxy-sparse"' \
        '' \
        '[source.rsproxy-sparse]' \
        'registry = "sparse+https://rsproxy.cn/index/"' \
        '' \
        '[net]' \
        'git-fetch-with-cli = true' \
        > /usr/local/cargo/config.toml; \
    fi
RUN rustup show active-toolchain | awk '{print $1}' > /tmp/rust-toolchain-docker
COPY . .
RUN mkdir -p /app/content
RUN RUSTUP_TOOLCHAIN="$(cat /tmp/rust-toolchain-docker)" cargo build --release -p stellartrail-api

FROM rust:1.95.0-bookworm AS runtime
WORKDIR /app
ARG APP_COMMIT_HASH=
ENV APP_COMMIT_HASH=$APP_COMMIT_HASH
COPY --from=builder /app/target/release/stellartrail-api /usr/local/bin/stellartrail-api
COPY --from=builder /app/content /app/content
RUN test -f /etc/ssl/certs/ca-certificates.crt
EXPOSE 8080
CMD ["stellartrail-api"]
