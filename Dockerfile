ARG BASE_IMAGE=dolphinjiang/rust-musl-builder:latest
FROM ${BASE_IMAGE} AS builder
ADD --chown=rust:rust . ./
RUN apk update && apk add gcc curl git file
RUN git clone https://github.com/jlaurens/synctex.git && cd synctex && gcc -c -fPIC *.c && gcc -shared *.o -o libsynctex_parser.so -lz
RUN cp libsynctex_parser.so ../src/so
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
RUN apk update && apk add curl websocat
CMD ["./texhub-server"]