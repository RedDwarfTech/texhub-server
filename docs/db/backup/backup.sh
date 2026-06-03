# 环境变量（通过 Kubernetes Secret 注入）
BACKUP_DIR="/backup"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="${POSTGRES_DB}"
BACKUP_FILE="${BACKUP_DIR}/${DB_NAME}_${DATE}.sql.gz"

echo "Starting backup of database: ${DB_NAME}"

# 执行 pg_dump 并压缩
pg_dump -h "${POSTGRES_HOST}" \
        -U "${POSTGRES_USER}" \
        -d "${DB_NAME}" \
        -p "${POSTGRES_PORT:-5432}" \
        | gzip > "${BACKUP_FILE}"

# 可选：保留最近 7 天的备份
find "${BACKUP_DIR}" -name "*.sql.gz" -mtime +7 -delete

echo "Backup completed: ${BACKUP_FILE}"