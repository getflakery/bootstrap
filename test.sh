#!/usr/bin/env bash

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
      \"port\": 8888,
      \"health_check_enabled\": true,
      \"health_check_path\": \"/\"
    }
  ],
  \"files\" : [
    {
      \"path\": \"/tsauthkey\",
      \"content\": \"`sudo cat /tsauthkey`\"
    }
  ],
  \"template_id\": \"c9c185b0-8b77-4f15-8ffa-7bada35fe48d\"
}"