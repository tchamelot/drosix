#!/bin/sh
set -eE # Exit on any error

runner="root@192.168.7.2"

error_handler() {
    case $1 in
        15) echo "Error could not connect to $runner";;
        *) echo Unknown error line $@;;
    esac
}
trap 'error_handler $LINENO' ERR

upload() {
    scp -q -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no \
        -o ConnectTimeout=1 \
        -i .cargo/id_rsa \
        $1 $runner: 2>/dev/null
}

run() {
    local target=$(basename $1)
    shift
    ssh -q -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no \
        -o ConnectTimeout=1 \
        -i .cargo/id_rsa \
        $runner ./$target $@
}

upload $1
run $@
