#! /usr/bin/env nix-shell
#! nix-shell -i sh -p awscli2 jq coreutils ripgrep util-linux


default_flags="--no-cli-auto-prompt --no-cli-pager"


# idiomatic parameter and option handling in sh
while test $# -gt 0
do
    case "$1" in
        --build) nix build .#ami
            ;;
        --cp) aws s3 cp "result/nixos-amazon-image-23.11.20231129.057f9ae-x86_64-linux.vhd"  "s3://nixos-base/bootstrap/nixos-bootstrap-debug.vhd"
            ;;
        *) echo "argument $1"
            ;;
    esac
    shift
done


itid=$(aws ec2 import-snapshot $default_flags \
    --description "flakery nixos bootstrap" \
    --disk-container "file://flakery-base/containers-debug.json" |  jq .ImportTaskId)
without_quotes=`echo $itid | sed 's/"//g'`


# Watch the progress of an import snapshot task and wait for completion
while true; do
    status=$(aws ec2 describe-import-snapshot-tasks --import-task-ids $without_quotes | jq -r '.ImportSnapshotTasks[0].SnapshotTaskDetail.Status')
    message=$(aws ec2 describe-import-snapshot-tasks --import-task-ids $without_quotes | jq -r '.ImportSnapshotTasks[0].SnapshotTaskDetail.StatusMessage')
    echo "Status: $status"
    if [[ "$status" == "completed" ]]; then
        echo "Snapshot import completed."
        break
    else
        echo "Waiting for snapshot to complete."
        echo "Current status: $status"
        echo "Message: $message"
        sleep 5
    fi
done


snapshot_id=$(aws ec2 describe-import-snapshot-tasks $default_flags --import-task-ids $without_quotes | jq -r '.ImportSnapshotTasks[0].SnapshotTaskDetail.SnapshotId')
si_without_quotes=`echo $snapshot_id | sed 's/"//g'`

echo "Snapshot ID: $si_without_quotes"

aws ec2 register-image \
    $default_flags \
    --name "flakery-nixos-`uuidgen`" \
    --root-device-name "/dev/xvda" \
    --block-device-mappings "[{\"DeviceName\":\"/dev/xvda\",\"Ebs\":{\"SnapshotId\":\"$si_without_quotes\"}}]"  \
    --architecture x86_64 --virtualization-type hvm --ena-support
