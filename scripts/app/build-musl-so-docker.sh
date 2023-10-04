#!/usr/bin/env bash

set -u

set -e

set -x

docker build -f ./ci/musl-so-test.dockerfile -t="reddwarf-pro/so-test-server:v1.0.0" .