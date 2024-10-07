#!/usr/bin/env bash

set -u

set -e

set -x

#backup structure
/opt/homebrew/bin/pg_dump -v -h reddwarf-postgresql.reddwarf-storage.svc.cluster.local \
-U postgres -p 5432 -s -d tex \
-f /Users/xiaoqiangjiang/source/reddwarf/backend/texhub-server/scripts/sql/tex-$(date '+%Y%m%d%H%M%S').sql

# backup structure and data
/opt/homebrew/bin/pg_dump -v -h reddwarf-postgresql.reddwarf-storage.svc.cluster.local \
-U postgres -p 5432 -d tex \
-f /Users/xiaoqiangjiang/backup/texhub/tex-$(date '+%Y%m%d%H%M%S').sql