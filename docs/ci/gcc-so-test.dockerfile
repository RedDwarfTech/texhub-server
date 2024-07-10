# this dockerfile used to test building c so file under gcc
ARG BASE_IMAGE=rust:1.72-bullseye
FROM ${BASE_IMAGE} AS builder
WORKDIR /app
COPY . /app
RUN rustup default stable


COPY ../scripts/shell/entrypoint.sh /entrypoint.sh
CMD ["/entrypoint.sh"]