# Build the application from source.
FROM rust:1.81.0-slim@sha256:3e24ad2190a3a1b91532478cd48b4e36fdcfc0eb4ee89a34e064c145febe360c AS rust-builder

ENV CARGO_HOME="/cache/cargo"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:debug@sha256:9678ecb30354b5e862a6dcaa075681d7e5df00a8899b473e4475c0834f5a1682 AS runtime
LABEL maintainer="DeadNews <deadnewsgit@gmail.com>"

ENV SERVICE_PORT=8000

COPY --from=rust-builder /app/target/release/deadnews-template-rust /bin/deadnews-template-rust

RUN ["/busybox/sh", "-c", "ln -s /busybox/sh /bin/sh"]

USER nonroot:nonroot
EXPOSE ${SERVICE_PORT}
HEALTHCHECK NONE

ENTRYPOINT ["/bin/deadnews-template-rust"]
