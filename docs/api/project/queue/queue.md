


```bash
curl -i -L -X POST -H "Content-Type: application/json" -d '{"docId": "12345", "projectId": "1749a976207049d3959", "initContext": "Hello"}' http://localhost:8000/inner-tex/queue/expire-check

curl -i -v \
  --noproxy "localhost,127.0.0.1" \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"docId": "12345", "projectId": "1749a976207049d3959", "initContext": "Hello"}' \
  http://localhost:8000/inner-tex/queue/expire-check
```
