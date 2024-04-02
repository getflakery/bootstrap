# testing 

```
{
  "flake_url": "github:r33drichards/go-webserver#flakery",
  "instance_type": "t3.small",
  "deployment_slug": "flakery-test",
  "subdomain_prefix": "flakery-test",
  "min_size": 1,
  "max_size": 1,
  "targets": [
    {
      "port": 8080,
      "health_check_enabled": false
    }
  ]
}
```

```bash
#!/bin/bash

# Generate a unique deployment slug by extracting the first 6 characters of a UUID
slug=flakery-$(uuidgen | grep -o '^......')

# Use the generated slug in the curl command with proper string substitution
curl -X 'POST' \
  'http://0.0.0.0:8000/deploy/aws/create' \
  -H 'accept: application/json' \
  -H 'Content-Type: application/json' \
  -d "{
  \"flake_url\": \"github:r33drichards/go-webserver#flakery\",
  \"instance_type\": \"t3.small\",
  \"deployment_slug\": \"${slug}\",
  \"subdomain_prefix\": \"${slug}\",
  \"min_size\": 1,
  \"max_size\": 1,
  \"targets\": [
    {
      \"port\": 8080,
      \"health_check_enabled\": true,
      \"health_check_path\": \"/\"
    }
  ],
  \"files\" : [
    {
      \"path\": \"/tsauthkey\",
      \"content\": \"`sudo cat /tsauthkey`\"
    }
  ]
}"

```
http://0.0.0.0:8000/swagger-ui/index.html



# building the base image 

```
nix build .#ami
aws s3 cp  result/nixos-amazon-image-23.11.20240326.4473351-x86_64-linux.vhd s3://nixos-base/bootstrap/nixos-amazon-image-23.11.20240326.4473351-x86_64-linux.vhd
aws ec2 import-snapshot --no-cli-auto-prompt --no-cli-pager --description "flakery nixos bootstrap" --disk-container "file://flakery-base/containers.json"   
```

```
{
    "Description": "flakery nixos",
    "ImportTaskId": "import-snap-01c750a9b69d61f1e",
    "SnapshotTaskDetail": {
        "Description": "flakery nixos",
        "DiskImageSize": 0.0,
        "Progress": "0",
        "Status": "active",
        "StatusMessage": "pending",
        "Url": "s3://nixos-base/nixos-amazon-image-23.11.20240316.8ac30a3-x86_64-linux.vhd"
    },
    "Tags": []
}
```


```
watch "aws ec2 describe-import-snapshot-tasks --import-task-ids import-snap-07050a7e8ed83e73b"  
```

```

{
    "ImportSnapshotTasks": [
        {
            "Description": "flakery nixos",
            "ImportTaskId": "import-snap-01c750a9b69d61f1e",
            "SnapshotTaskDetail": {
                "Description": "flakery nixos",
                "DiskImageSize": 1688628224.0,
                "Format": "VHD",
                "SnapshotId": "snap-0523cd0d0571f5e48",
                "Status": "completed",
                "Url": "s3://nixos-base/nixos-amazon-image-23.11.20240316.8ac30a3-x86_64-linux.vhd
",
                "UserBucket": {
                    "S3Bucket": "nixos-base",
                    "S3Key": "nixos-amazon-image-23.11.20240316.8ac30a3-x86_64-linux.vhd"
                }
            },
            "Tags": []
        }
    ]
}
```


# integration testing 
```
nix build -L .#test.driverInteractive && ./result/bin/nixos-test-driver
nix build -L .#test
```

```
ssh root@localhost -p2222 -o StrictHostKeyChecking=no
```