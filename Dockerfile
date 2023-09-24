# Build the application from source.
FROM rust:1.72.1-slim@sha256:ff798ceb500fa43c91db10db881066057fefd36e16d531e7b1ed228ab2246175 as rust-builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo build --release --locked

# Deploy the application binary into a lean image.
FROM gcr.io/distroless/cc-debian12:latest@sha256:f44927808110f578fba42bf36eb68a5ecbb268b94543eb9725380ec51e9a39ed AS runtime
LABEL maintainer "DeadNews <aurczpbgr@mozmail.com>"

COPY --from=rust-builder /app/target/release/deadnews-template-rust /usr/local/bin/deadnews-template-rust

USER nonroot:nonroot
EXPOSE 1271
HEALTHCHECK --interval=60s --timeout=3s CMD curl --fail http://127.0.0.1:1271/health || exit 1

CMD ["deadnews-template-rust"]
