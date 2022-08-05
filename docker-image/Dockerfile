# Container image that runs your code
FROM rust as build
RUN cargo install filesfinder

FROM debian:buster-slim
COPY --from=build /usr/local/cargo/bin/ff /usr/local/bin/ff

CMD [ "ff", "*" ]
