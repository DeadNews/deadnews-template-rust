# Build the application from source.
FROM rust:1.73.0-slim@sha256:89e1efffc83a631bced1bf86135f4f671223cc5dc32ebf26ef8b3efd1b97ffff as rust-builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Maunt as dedicated RUN cache.
ENV CARGO_HOME="/cache/cargo"
RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:latest@sha256:88ab13ee3757ecf86ebf1b0dcdb8402c21fd977afb55cd68f6a3d334bdaea750 AS runtime
LABEL maintainer "DeadNews <aurczpbgr@mozmail.com>"

COPY --from=rust-builder /app/target/release/deadnews-template-rust /usr/local/bin/deadnews-template-rust

USER nonroot:nonroot
EXPOSE 8080
HEALTHCHECK NONE

CMD ["deadnews-template-rust"]
