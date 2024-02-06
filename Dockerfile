FROM rust:latest AS build
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install --path . --target x86_64-unknown-linux-musl

FROM alpine:3.16.0 AS runtime
COPY --from=build /usr/local/cargo/bin/ff /usr/local/bin/ff

FROM runtime as action
COPY entrypoint.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
