#!/usr/bin/env bash

set -u

set -e

set -x

/opt/homebrew/bin/pg_dump -v -h reddwarf-postgresql.reddwarf-storage.svc.cluster.local \
-U postgres -p 5432 -d cv \
-f /Users/xiaoqiangjiang/backup/cv-$(date '+%Y%m%d%H%M%S').sql