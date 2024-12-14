# Build the application from source.
FROM rust:1.83.0-slim@sha256:c5bf976be6d358b7dc6113fe0ef179077244dff8fdd9c3bec1bcd14677d1f902 AS rust-builder

ENV CARGO_HOME="/cache/cargo"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:debug@sha256:c1aad82df2b344ef00c9d01608f6ef13b833e50be2affb63fb4e87fe03df46e4 AS runtime
LABEL maintainer="DeadNews <deadnewsgit@gmail.com>"

ENV SERVICE_PORT=8000

COPY --from=rust-builder /app/target/release/deadnews-template-rust /bin/deadnews-template-rust

RUN ["/busybox/sh", "-c", "ln -s /busybox/sh /bin/sh"]

USER nonroot:nonroot
EXPOSE ${SERVICE_PORT}
HEALTHCHECK NONE

ENTRYPOINT ["/bin/deadnews-template-rust"]
