# Build the application from source.
FROM rust:1.82.0-slim@sha256:9abf10cc84dfad6ace1b0aae3951dc5200f467c593394288c11db1e17bb4d349 AS rust-builder

ENV CARGO_HOME="/cache/cargo"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:debug@sha256:1571f6d92cd6b3303bc728736113f6e4ed648e2976e4d875113eb0fba581522f AS runtime
LABEL maintainer="DeadNews <deadnewsgit@gmail.com>"

ENV SERVICE_PORT=8000

COPY --from=rust-builder /app/target/release/deadnews-template-rust /bin/deadnews-template-rust

RUN ["/busybox/sh", "-c", "ln -s /busybox/sh /bin/sh"]

USER nonroot:nonroot
EXPOSE ${SERVICE_PORT}
HEALTHCHECK NONE

ENTRYPOINT ["/bin/deadnews-template-rust"]
