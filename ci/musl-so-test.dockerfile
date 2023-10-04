# this dockerfile used to test building c so file under musl
ARG BASE_IMAGE=dolphinjiang/rust-musl-builder:latest
FROM ${BASE_IMAGE} AS builder
ADD --chown=rust:rust . ./
COPY ../scripts/shell/entrypoint.sh /entrypoint.sh
CMD ["/entrypoint.sh"]
