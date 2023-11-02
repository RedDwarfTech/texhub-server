
### 发送查询

```bash
curl 'http://localhost:7700/indexes/files/search' \
    -X GET \
    --header 'Content-Type: application/json' \
    --header 'Authorization: Bearer master-key' \
    --data-raw '{
        "q": "ability",
        "limit": 20,
        "offset": 0
    }'
```






