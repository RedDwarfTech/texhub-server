ARG BASE_IMAGE=rust:1.54-bullseye
FROM ${BASE_IMAGE} AS builder
ADD --chown=rust:rust . ./
RUN RUSTFLAGS='-L ./src/so' cargo build --release

FROM debian:bullseye-slim
LABEL maintainer="jiangtingqiang@gmail.com"
WORKDIR /app
ENV ROCKET_ADDRESS=0.0.0.0
COPY --from=builder /home/rust/src/settings.toml /app
COPY --from=builder /home/rust/src/log4rs.yaml /app
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/texhub-server /app/
RUN apk update && apk add curl websocat
CMD ["./texhub-server"]