# Container image that runs your code
FROM rust:1.61.0-slim AS build
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install filesfinder --target x86_64-unknown-linux-musl

FROM alpine:3.16.0 AS runtime 
COPY --from=build /usr/local/cargo/bin/ff /usr/local/bin/ff

CMD [ "ff", "*" ]
