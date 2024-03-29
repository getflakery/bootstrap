```
[root@ip-10-0-0-184:~]# curl http://169.254.169.254/latest/meta-data/tags/instance
Name
foo
[root@ip-10-0-0-184:~]# curl http://169.254.169.254/latest/meta-data/tags/instance/foo
bar
```

decrypt files with 
https://docs.rs/libaes/latest/libaes/
https://docs.rs/scrypt/latest/scrypt/