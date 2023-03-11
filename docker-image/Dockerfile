# Container image that runs your code
FROM rust:1.67 AS build
RUN git clone https://github.com/jeertmans/filesfinder.git
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo install --path filesfinder --target x86_64-unknown-linux-musl

FROM alpine:3.16.0 AS runtime
COPY --from=build /usr/local/cargo/bin/ff /usr/local/bin/ff

CMD [ "ff", "*" ]
