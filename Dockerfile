# Build the application from source.
FROM rust:1.78.0-slim@sha256:0fea967628dc796a2b9d1d57ddb3af3b3f0a35b6c8c0e23690dbe0ceb71a2dc9 as rust-builder

ENV CARGO_HOME="/cache/cargo"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:latest@sha256:e1065a1d58800a7294f74e67c32ec4146d09d6cbe471c1fa7ed456b2d2bf06e0 AS runtime
LABEL maintainer "DeadNews <deadnewsgit@gmail.com>"

COPY --from=rust-builder /app/target/release/deadnews-template-rust /usr/local/bin/deadnews-template-rust

USER nonroot:nonroot
EXPOSE 8080
HEALTHCHECK NONE

CMD ["deadnews-template-rust"]
