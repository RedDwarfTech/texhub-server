ARG BASE_IMAGE=dolphinjiang/rust-musl-builder:latest
FROM ${BASE_IMAGE} AS builder
ADD --chown=rust:rust . ./
RUN cargo build --release

FROM alpine:3.18.2
LABEL maintainer="jiangtingqiang@gmail.com"
WORKDIR /app
ENV ROCKET_ADDRESS=0.0.0.0
COPY --from=builder /home/rust/src/settings.toml /app
COPY --from=builder /home/rust/src/scripts /app/
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/rss-sync /app/
RUN apk update && apk add curl
CMD ["sh","./startup-app.sh"]