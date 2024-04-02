#!/usr/bin/env bash
echo 'import id: ' $1
echo 'waiting for image to be imported'
aws ec2 wait snapshot-imported --import-task-ids $1 --region us-west-1 

echo 'registering image' 
aws ec2 register-image --name "NixOS-$(date +%Y%m%d)" --architecture x86_64 --root-device-name /dev/sda1 --block-device-mappings "DeviceName=/dev/sda1,Ebs=`aws ec2 describe-import-snapshot-tasks --import-task-ids import-snap-025b351f808c91516  --no-cli-pager | jq '.ImportSnapshotTasks[0].SnapshotTaskDetail.SnapshotId' | sed 's/"//g' | sed 's/ +//g'`" --virtualization-type hvm --region us-west-1
