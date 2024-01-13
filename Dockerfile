# Build the application from source.
FROM rust:1.75.0-slim@sha256:52d43338714f1939b0a54f2004f76b7cd0e7cb0e8297c909a2f54c1942f990ff as rust-builder

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
