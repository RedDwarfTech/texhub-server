

### 请求编译项目

```bash
# 直接请求渲染器
curl -N -H "Accept: text/event-stream" http://cv-render-service.reddwarf-pro.svc.cluster.local:8000/render/compile/v1/project/sse\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_name=main.tex\&file_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1/main.tex\&out_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1

# 内部请求渲染器
curl -N -H "Accept: text/event-stream" -H "content-type: application/json" http://localhost:8000/render/compile/v1/project/sse\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1/main.tex\&out_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1

# 内部请求服务器
curl -N -H "Accept: text/event-stream" -H "content-type: application/json" http://localhost:8000/tex/project/log/stream\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_name=main.tex\&tac=123456

# 远程服务器请求，服务中请求可以流式返回，远程服务器请求直接从宿主机中发起，判断哪里导致流式返回失效
curl -N -H "Accept: text/event-stream" -H "Host: tex.poemhub.top" -H "content-type: application/json" http://127.0.0.1:8000/tex/project/log/stream\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_name=main.tex\&tac=123456

# 远程服务器请求，通过域名发起
curl -N -H "Accept: text/event-stream" -H "Host: tex.poemhub.top" -H "content-type: application/json" https://tex.poemhub.top/tex/project/log/stream\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_name=main.tex\&tac=123456
```


