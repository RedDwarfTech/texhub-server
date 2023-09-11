### 项目

#### 请求编译项目

```bash
# 直接请求渲染器
curl -N -H "Accept: text/event-stream" http://cv-render-service.reddwarf-pro.svc.cluster.local:8000/render/compile/v1/project/sse\?project_id=c98f73bc869143d084eb38a0fc38a8e7\&req_time=1693498954408\&file_name=main.tex\&file_path=/opt/data/project/c98f73bc869143d084eb38a0fc38a8e7/main.tex\&out_path=/opt/data/project/c98f73bc869143d084eb38a0fc38a8e7

# 内部请求渲染器
curl -N -H "Accept: text/event-stream" -H "content-type: application/json" http://localhost:8000/render/compile/v1/project/sse\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1/main.tex\&out_path=/opt/data/project/5007e247d30d4bd6beaa72af1d9124c1

# 内部请求服务器
curl -N -H "Accept: text/event-stream" -H "content-type: application/json" http://localhost:8000/tex/project/log/stream\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_name=main.tex\&tac=123456

# 远程服务器请求，服务中请求可以流式返回，远程服务器请求直接从宿主机中发起，判断哪里导致流式返回失效
curl -N -H "Accept: text/event-stream" -H "Host: tex.poemhub.top" -H "content-type: application/json" http://127.0.0.1:8000/tex/project/log/stream\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_name=main.tex\&tac=123456

# 远程服务器请求，通过域名发起
curl -N -H "Accept: text/event-stream" -H "Host: tex.poemhub.top" -H "content-type: application/json" https://tex.poemhub.top/tex/project/log/stream\?project_id=5007e247d30d4bd6beaa72af1d9124c1\&req_time=1693498954408\&file_name=main.tex\&tac=123456
```


#### 获取授权码

```
curl http://localhost:8000/tex/project/temp/code
```

#### 请求SSE编译日志


```
curl 'http://localhost:8000/tex/project/compile/qlog?project_id=c98f73bc869143d084eb38a0fc38a8e7&file_name=main.tex&version_no=f49741af436644e9bed0fd17753c6007' \
  -H 'Accept: text/event-stream' \
  -H 'Accept-Language: en,zh-CN;q=0.9,zh;q=0.8,zh-TW;q=0.7,fr;q=0.6' \
  -H 'Cache-Control: no-cache' \
  -H 'Connection: keep-alive' \
  -H 'DNT: 1' \
  -H 'Referer: https://tex.poemhub.top/editor?pid=c98f73bc869143d084eb38a0fc38a8e7' \
  -H 'Sec-Fetch-Dest: empty' \
  -H 'Sec-Fetch-Mode: cors' \
  -H 'Sec-Fetch-Site: same-origin' \
  -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36' \
  -H 'sec-ch-ua: "Chromium";v="116", "Not)A;Brand";v="24", "Google Chrome";v="116"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  --compressed
```

#### 初始化模版内容到yjs


```bash
curl -X POST -H "Content-Type: application/json" -d '{"docId": "12345", "projectId": "67890", "initContext": "Hello"}' http://localhost:3000/y-websocket/file/initial
```



