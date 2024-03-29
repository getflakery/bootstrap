```
[root@ip-10-0-0-184:~]# curl http://169.254.169.254/latest/meta-data/tags/instance
Name
foo
[root@ip-10-0-0-184:~]# curl http://169.254.169.254/latest/meta-data/tags/instance/foo
bar
```

CRYPTO_STRING_KEY: process.env.CRYPTO_STRING_KEY || 'b355b95e-1933-4103-8f7e-156687fa0a1f',
CRYPTO_STRING_SALT: process.env.CRYPTO_STRING_SALT || 'b355b95e-1933-4103-8f7e-156687fa0a1f',

decrypt files with 
https://docs.rs/libaes/latest/libaes/
https://docs.rs/scrypt/latest/scrypt/