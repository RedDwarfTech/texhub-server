
Clone项目

```bash
curl -X POST -H 'authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VySWQiOjEwMywiZGV2aWNlSWQiOiJjNTg5ODFlOGY4ODAwZDk1YjM0Mzk0YTViZGVlYzJhOSIsImFwcElkIjoibjI5UGEyOVdTMSIsImx0IjoxLCJldCI6MTczMTUwNDQ4OTQyNywicGlkIjoxMywiZXhwIjoxNzI5MzQ5OTM2fQ.wvbzBT-44BTK8Jf3I7ZMLrXf22hkL8HkJ_WdN9JEejY' http://127.0.0.1:8000/tex/project/github/import --data-raw '{"url":"https://github.com/jiangxiaoqiang/dolphin-book-2020.git","main_file":"dolphin-book-2020.tex"}'
```

Clone公开项目：


```bash
curl -X POST -H 'authorization:Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VySWQiOjEwMywiZGV2aWNlSWQiOiJjNTg5ODFlOGY4ODAwZDk1YjM0Mzk0YTViZGVlYzJhOSIsImFwcElkIjoibjI5UGEyOVdTMSIsImx0IjoxLCJldCI6MTczMTUwNDQ4OTQyNywicGlkIjoxMywiZXhwIjoxNzI5MzQ5OTM2fQ.wvbzBT-44BTK8Jf3I7ZMLrXf22hkL8HkJ_WdN9JEejY' http://127.0.0.1:8000/tex/project/github/import --data-raw '{"url":"https://github.com/jiangxiaoqiang/devmanual.git","main_file":"document.tex"}'
```