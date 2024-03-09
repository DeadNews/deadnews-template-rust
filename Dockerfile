# Build the application from source.
FROM rust:1.76.0-slim@sha256:de22cea71b620c7fdc61e8c1bf3f048d0ffbafe062ca9d7b32aed6a7d59109a4 as rust-builder

ENV CARGO_HOME="/cache/cargo"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:latest@sha256:efafe74d452c57025616c816b058e3d453c184e4b337897a8d38fef5026b079d AS runtime
LABEL maintainer "DeadNews <aurczpbgr@mozmail.com>"

COPY --from=rust-builder /app/target/release/deadnews-template-rust /usr/local/bin/deadnews-template-rust

USER nonroot:nonroot
EXPOSE 8080
HEALTHCHECK NONE

CMD ["deadnews-template-rust"]
