查询项目信息：

```Bash
curl -X GET -H 'authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VySWQiOjEwMywiZGV2aWNlSWQiOiI2NjgxMmMwNTgwZmM3N2FjNThkZWFkNTBkMDQwMmYwMyIsImFwcElkIjoibjI5UGEyOVdTMSIsImx0IjoxLCJldCI6MCwicGlkIjoxMywiZXhwIjoxNzUzNTEyMDU2fQ.Uq8OMbM9gUjaWp0yfxjQ9UsIFsvI7ZbPsz3hyHyCC9w' http://127.0.0.1:8000/tex/project/info?project_id=287ea73d1c78463298e9e5b267cab87f
```

### 查询位置信息

```bash
curl http://cv-render-service.reddwarf-pro.svc.cluster.local:8001/tex/project/pos/pdf?project_id=287ea73d1c78463298e9e5b267cab87f\&path=%2Fproject%2Ftexhub%2Flearn\&file=pdf-tab-switch.tex\&main_file=main.tex\&line=7\&column=11\&created_time=1694341283521
```

```bash
curl http://cv-render-service.reddwarf-pro.svc.cluster.local:8000/texhub/actuator/liveness
curl http://localhost:8001/texhub/actuator/liveness
```
