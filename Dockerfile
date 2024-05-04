# Build the application from source.
FROM rust:1.78.0-slim@sha256:517c6272b328bc51c87e099ef4adfbc7ab4558af2d757e8d423c7c3f1cbbf9d5 as rust-builder

ENV CARGO_HOME="/cache/cargo"

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN --mount=type=cache,target=${CARGO_HOME} \
    cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:latest@sha256:eed8bd290a9f83d0451e7812854da87a8407f1d68f44fae5261c16556be6465b AS runtime
LABEL maintainer "DeadNews <deadnewsgit@gmail.com>"

COPY --from=rust-builder /app/target/release/deadnews-template-rust /usr/local/bin/deadnews-template-rust

USER nonroot:nonroot
EXPOSE 8080
HEALTHCHECK NONE

CMD ["deadnews-template-rust"]
