#!/usr/bin/env bash

set -u
set -e
set -x

# https://stackoverflow.com/questions/71637346/option-to-get-the-rust-project-dir-in-shell-script
PROJECT_DIR="$(dirname "$(cargo locate-project|jq -r .root)")"

diesel --database-url postgres://postgres:${CV_POSTGRESQL_PWD}@reddwarf-postgresql.reddwarf-storage.svc.cluster.local:5432/tex \
migration run --config-file="${PROJECT_DIR}"/scripts/diesel/diesel-tex.toml
