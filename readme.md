# integration testing 
```
 ssh root@100.101.243.23 "nix build --extra-experimental-features 'nix-command flakes' --refresh -L github:getflakery/bootstrap#test.driverInteractive && ./result/bin/nixos-test-driver"
 
nix build -L .#test.driverInteractive && ./result/bin/nixos-test-driver

nix build -L .#test

ssh root@100.101.243.23 "nix build --extra-experimental-features 'nix-command flakes' --refresh -L github:getflakery/bootstrap#test"
```

```
ssh root@localhost -p2222 -o StrictHostKeyChecking=no
ssh -J root@100.101.243.23  root@localhost -p 2222 -o StrictHostKeyChecking=no    
```

# switch to bootstrap configuration
```
nixos-rebuild switch --flake .#bootstrap
nixos-rebuild switch --flake github:getflakery/bootstrap#bootstrap --refresh

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