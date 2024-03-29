```
[root@ip-10-0-0-184:~]# curl http://169.254.169.254/latest/meta-data/tags/instance
Name
foo
[root@ip-10-0-0-184:~]# curl http://169.254.169.254/latest/meta-data/tags/instance/foo
bar
```
