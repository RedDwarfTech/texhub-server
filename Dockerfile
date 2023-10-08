ARG BASE_IMAGE=dolphinjiang/rust-musl-builder:latest
FROM ${BASE_IMAGE} AS builder
ADD --chown=rust:rust . ./
RUN git clone --depth 1 --branch 91d69b73e2fc9c65953c04debe0f06fbd1e51299 https://github.com/jlaurens/synctex.git
RUN cd synctex && gcc -c -fPIC *.c && gcc -shared *.o -o libsynctex_parser.so -lz
RUN cp libsynctex_parser.so ../src/so/
RUN RUSTFLAGS='-L ./src/so' cargo build --release

FROM alpine:3.18.2
LABEL maintainer="jiangtingqiang@gmail.com"
WORKDIR /app
ENV ROCKET_ADDRESS=0.0.0.0
COPY --from=builder /home/rust/src/settings.toml /app
COPY --from=builder /home/rust/src/src/so/libsynctex_parser.so /app
COPY --from=builder /home/rust/src/src/so/libsynctex_parser.so /usr/lib/
COPY --from=builder /home/rust/src/log4rs.yaml /app
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/texhub-server /app/
RUN apk update && apk add curl websocat zlib zlib-dev openssl-dev openssl
CMD ["./texhub-server"]