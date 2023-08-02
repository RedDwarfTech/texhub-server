#!/usr/bin/env bash

set -u
set -e
set -x

PROJECT_DIR="$(dirname "$(cargo locate-project|jq -r .root)")"

diesel_ext --derive Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone \
--map "Jsonb serde_json::Value" \
--add-table-name \
--import-types "serde::Serialize" \
--import-types "serde::Deserialize" \
--import-types "crate::model::diesel::tex::tex_schema::*" \
--schema-file "${PROJECT_DIR}"/src/model/diesel/tex/tex_schema.rs --model > "${PROJECT_DIR}"/src/model/diesel/tex/tex_models.rs

