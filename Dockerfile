# Build the application from source.
FROM rust:1.74.1-slim@sha256:8f7df8eb8f5fc25284cb83a0ba6088a09c7d09490237f3393c62e8408491a6e6 as rust-builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Maunt as dedicated RUN cache.
ENV CARGO_HOME="/cache/cargo"
RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:latest@sha256:6714977f9f02632c31377650c15d89a7efaebf43bab0f37c712c30fc01edb973 AS runtime
LABEL maintainer "DeadNews <aurczpbgr@mozmail.com>"

COPY --from=rust-builder /app/target/release/deadnews-template-rust /usr/local/bin/deadnews-template-rust

USER nonroot:nonroot
EXPOSE 8080
HEALTHCHECK NONE

CMD ["deadnews-template-rust"]
