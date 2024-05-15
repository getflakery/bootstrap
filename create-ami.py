#! /usr/bin/env nix-shell
#! nix-shell -i python3 -p "python3.withPackages(ps: with ps; [ boto3 botocore ])"

import argparse
import boto3
import json
import time
import uuid
import os
import subprocess
from botocore.config import Config


def import_snapshot(description, disk_container):
    ec2_client = boto3.client('ec2', config=my_config)
    response = ec2_client.import_snapshot(
        Description=description,
        DiskContainer=disk_container
    )
    return response['ImportTaskId']

def get_snapshot_status(task_id):
    ec2_client = boto3.client('ec2', config=my_config)
    response = ec2_client.describe_import_snapshot_tasks(ImportTaskIds=[task_id])
    status = response['ImportSnapshotTasks'][0]['SnapshotTaskDetail']['Status']
    message = response['ImportSnapshotTasks'][0]['SnapshotTaskDetail']['StatusMessage']
    return status, message

def get_snapshot_id(task_id):
    ec2_client = boto3.client('ec2', config=my_config)
    response = ec2_client.describe_import_snapshot_tasks(ImportTaskIds=[task_id])
    snapshot_id = response['ImportSnapshotTasks'][0]['SnapshotTaskDetail']['SnapshotId']
    return snapshot_id

def register_image(snapshot_id):
    ec2_client = boto3.client('ec2', config=my_config)
    image_name = f"flakery-nixos-{uuid.uuid4()}"
    response = ec2_client.register_image(
        Name=image_name,
        RootDeviceName='/dev/xvda',
        BlockDeviceMappings=[
            {
                'DeviceName': '/dev/xvda',
                'Ebs': {
                    'SnapshotId': snapshot_id
                }
            }
        ],
        Architecture='x86_64',
        VirtualizationType='hvm',
        EnaSupport=True
    )
    print(f"Image registered with name: {image_name}")
    return response['ImageId']

# result/nixos-amazon-image-23.11.20231129.057f9ae-x86_64-linux.vhd
def get_result_path():
    out = os.listdir("result")
    # get the file name that starts with nixos-amazon-image and ends with .vhd
    for file in out:
        if file.startswith("nixos-amazon-image") and file.endswith(".vhd"):
            return f"result/{file}"
    
    raise Exception("No file found in result directory")


my_config = Config(
    region_name = 'us-west-2',
    signature_version = 'v4',
    retries = {
        'max_attempts': 10,
        'mode': 'standard'
    }
)

def main():
    parser = argparse.ArgumentParser(description="A script to handle AWS snapshot and image registration.")
    parser.add_argument("--flake", help="Build using nix", default=None)
    parser.add_argument("--s3-key", help="S3 key to upload the image to")

    
    args = parser.parse_args()

    if args.flake is not None: 
        subprocess.run(["nix", "build", args.flake])

        result_path = get_result_path()

        s3_client = boto3.client('s3', config=my_config)
        print("uploading to s3")
        print(f"Uploading {result_path} to s3://oofers/{args.s3_key}")
        s3_client.upload_file(
            result_path,
            'oofers',
            args.s3_key,
        )

    disk_container = {
        "Description": "NixOS Base",
        "Format": "vhd",
        "Url": f"s3://oofers/{args.s3_key}"
    }

    itid = import_snapshot("flakery nixos bootstrap", disk_container)
    print(f"Import Task ID: {itid}")

    while True:
        status, message = get_snapshot_status(itid)
        print(f"Status: {status}")
        if status == "completed":
            print("Snapshot import completed.")
            break
        else:
            print("Waiting for snapshot to complete.")
            print(f"Current status: {status}")
            print(f"Message: {message}")
            time.sleep(5)

    snapshot_id = get_snapshot_id(itid)
    print(f"Snapshot ID: {snapshot_id}")

    image_id = register_image(snapshot_id)
    print(f"Image ID: {image_id}")

if __name__ == "__main__":
    main()
