# Build the application from source.
FROM rust:1.77.1-slim@sha256:725959da2e3079f69c33ff2b9b0826d44912e45866cc89239b8700544946aefc as rust-builder

ENV CARGO_HOME="/cache/cargo"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:latest@sha256:7a01d633f75120af59c71489e0911fa8b6512673a3ff0b999522b4221ab4d86a AS runtime
LABEL maintainer "DeadNews <deadnewsgit@gmail.com>"

COPY --from=rust-builder /app/target/release/deadnews-template-rust /usr/local/bin/deadnews-template-rust

USER nonroot:nonroot
EXPOSE 8080
HEALTHCHECK NONE

CMD ["deadnews-template-rust"]
