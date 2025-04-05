# Build the application from source.
FROM rust:1.86.0-slim@sha256:9c1ef35ab804dc78361948794f60748e79a7a2e297580604b288590bc52ebdaa AS rust-builder

ENV CARGO_HOME="/cache/cargo"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:debug@sha256:5ccfee06c7ddc5aebcb7c0907d7d5346175f640200e906777259031674e70a37 AS runtime
LABEL maintainer="DeadNews <deadnewsgit@gmail.com>"

ENV SERVICE_PORT=8000

COPY --from=rust-builder /app/target/release/deadnews-template-rust /bin/deadnews-template-rust

RUN ["/busybox/sh", "-c", "ln -s /busybox/sh /bin/sh"]

USER nonroot:nonroot
EXPOSE ${SERVICE_PORT}
HEALTHCHECK NONE

ENTRYPOINT ["/bin/deadnews-template-rust"]
