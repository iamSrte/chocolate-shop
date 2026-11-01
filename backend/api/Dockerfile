# Build context: backend/. `docker build -f backend/api/Dockerfile backend`
FROM rust:1-slim-trixie AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY api api
COPY domain domain
COPY .sqlx .sqlx
ENV SQLX_OFFLINE=true
RUN cargo build --release --locked -p api --bin api

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 app
COPY --from=build /app/target/release/api /usr/local/bin/api
ENV API_ADDR=0.0.0.0:3000 LOG_FORMAT=json
EXPOSE 3000
USER app
CMD ["api"]
