ARG BASE_IMAGE=dolphinjiang/rust-musl-builder:latest
FROM ${BASE_IMAGE} AS builder
ADD --chown=rust:rust . ./
RUN RUSTFLAGS='-L ./src/so' cargo build --release

FROM alpine:3.18.2
LABEL maintainer="jiangtingqiang@gmail.com"
WORKDIR /app
ENV ROCKET_ADDRESS=0.0.0.0
RUN export LD_LIBRARY_PATH=/app:$LD_LIBRARY_PATH
COPY --from=builder /home/rust/src/settings.toml /app
COPY --from=builder /home/rust/src/src/so/libsynctex_parser.so /app
COPY --from=builder /home/rust/src/log4rs.yaml /app
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/texhub-server /app/
RUN apk update && apk add curl websocat zlib git gcc
CMD ["./texhub-server"]