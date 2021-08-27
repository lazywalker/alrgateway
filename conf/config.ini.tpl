[proxy]
hostname=localhost
local_ip=127.0.0.1
local_port=8000
remote_ip=10.10.20.39
jwt_audience=alr
jwt_secret=secret

[gw]
api_server=10.10.20.30:81
api_secret=**********

[db]
host=localhost
user=postgres
password=123654
dbname=postgres