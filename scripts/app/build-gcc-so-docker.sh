#!/usr/bin/env bash

set -u

set -e

set -x

docker build -f ./ci/gcc-so-test.dockerfile -t="reddwarf-pro/gcc-test-server:v1.0.0" .