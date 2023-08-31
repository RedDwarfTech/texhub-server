

### 请求编译

```bash
curl -N -H "Accept: text/event-stream" http://cv-render-service.reddwarf-pro.svc.cluster.local:8000/render/compile/v1/project/sse\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_name=main.tex\&file_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1/main.tex\&out_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1

curl -N -H "Accept: text/event-stream" -H "content-type: application/json" http://localhost:8000/render/compile/v1/project/sse?project_id=5007e247d30d4bd6beaa72af1d9124c1&req_time=1693498954408&file_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1/main.tex&out_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1
```


