FROM postgres:15-alpine

# 安装 bash（可选，alpine 默认有 sh）
RUN apk add --no-cache bash

# 创建备份目录
RUN mkdir -p /backup

COPY backup.sh /usr/local/bin/backup.sh
RUN chmod +x /usr/local/bin/backup.sh

ENTRYPOINT ["/usr/local/bin/backup.sh"]