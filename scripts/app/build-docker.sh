#!/usr/bin/env bash

set -u

set -e

set -x

docker build -f ./Dockerfile -t="reddwarf-pro/alt-server:v1.0.0" .