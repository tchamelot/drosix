#!/bin/sh
set -eE # Exit on any error

runner="root@192.168.6.1"

error_handler() {
    case $1 in
        15) echo "Error could not connect to $runner";;
        *) echo Unknown error line $@;;
    esac
}
trap 'error_handler $LINENO' EXIT

upload() {
    scp -q -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no \
        -o ConnectTimeout=1 \
        -i $(dirname $0)/id_rsa \
        $1 $runner:/tmp 2>/dev/null
}

run() {
    local target=$(basename $1)
    shift
    ssh -q -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no \
        -o ConnectTimeout=1 \
        -i $(dirname $0)/id_rsa \
        $runner /tmp/$target $@
}

upload $1
run $@
