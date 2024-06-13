# testing 

```json
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
}"

```
http://0.0.0.0:8000/swagger-ui/index.html



# building the base image 

```sh
nix build .#amiDebug
aws s3 cp  result/nixos-amazon-image-23.11.20231129.057f9ae-x86_64-linux.vhd  s3://nixos-base/bootstrap/nixos-bootstrap-debug.vhd
aws ec2 import-snapshot --no-cli-auto-prompt --no-cli-pager --description "flakery nixos bootstrap" --disk-container "file://flakery-base/containers-debug.json" | jq .ImportTaskId
```

```json
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


```sh
watch "aws ec2 describe-import-snapshot-tasks --import-task-ids import-snap-0a9724697e580e1fe"  
```
snap-0fd6c4840f8c3fc7e
```json

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

# snapshot to ami
snap-0ec9a792b5dd86ba8

```bash
aws ec2 register-image --name "flakery-nixos-testtT" --root-device-name "/dev/xvda" --block-device-mappings "[{\"DeviceName\":\"/dev/xvda\",\"Ebs\":{\"SnapshotId\":\"snap-04ccb9d509fd1358e\"}}]"  \
    --architecture x86_64 --virtualization-type hvm --ena-support
```

```
{
    "ImageId": "ami-081cdd79bd60a67b7"
}
```

# delete all autoscaling groups in region us-west-1
```bash
aws autoscaling describe-auto-scaling-groups --region us-west-1 | jq -r '.AutoScalingGroups[].AutoScalingGroupName' | xargs -I {} aws autoscaling delete-auto-scaling-group --auto-scaling-group-name {} --region us-west-1
```

# delete all alb's in region us-west-1
```bash
aws elbv2 describe-load-balancers --region us-west-1 | jq -r '.LoadBalancers[].LoadBalancerArn' | xargs -I {} aws elbv2 delete-load-balancer --load-balancer-arn {} --region us-west-1
```

# integration testing 
```
nix build -L .#test.driverInteractive && ./result/bin/nixos-test-driver
nix build -L .#test
```

```
ssh root@localhost -p2222 -o StrictHostKeyChecking=no
```

# switch to bootstrap configuration
```
nixos-rebuild switch --flake .#bootstrap
nixos-rebuild switch --flake github:getflakery/bootstrap#bootstrap --refresh

```

# start webserver 

```

```

# gen openapi 
```
nix develop --command cargo run --bin webserver -- --print-openapi > openapi.json
```

# gen client
```
openapi-generator-cli generate -i ./openapi.json -g rust -o /tmp/test/ 
```

# test an endpoint with fake 
```
curl -i -X POST -H "Debug: 1" -H "Content-Type: application/json" -d '{"deployment_id":"foo", "mappings": [{ "listener_port": 443,  "target_port": 8000}]}' http://localhost:8000/create-listener


curl -i -X POST -H "Content-Type: application/json" -d '{"deployment_id":"foo", "mappings": [{ "listener_port": 443,  "target_port": 8000}]}' http://localhost:8000/create-listener

```


# build debug ami 
```
./debug-ami.sh --build --cp
```

# delete lt

```
# List and delete launch templates with the prefix "flakery-"
aws ec2 describe-launch-templates --query "LaunchTemplates[?starts_with(LaunchTemplateName, 'flakery-')].[LaunchTemplateId]" --output text --region us-west-1 | while read -r template_id
do
    echo "Deleting launch template $template_id"
    aws ec2 delete-launch-template --launch-template-id "$template_id" --region us-west-1 --no-cli-pager
done
```


https://github.com/nix-community/nixos-generators/blob/35c20ba421dfa5059e20e0ef2343c875372bdcf3/formats/raw.nix#L25C28-L25C36