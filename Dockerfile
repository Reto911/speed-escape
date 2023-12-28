ARG RUST_IMAGE_VERSION=latest
FROM rust:${RUST_IMAGE_VERSION} as builder
WORKDIR /tmp/build
COPY . .
RUN cargo build

FROM ubuntu:latest

COPY --from=builder /tmp/build/target/debug/escape_speed /usr/local/bin/escape_speed
ENTRYPOINT ["/usr/local/bin/escape_speed"]
CMD []
