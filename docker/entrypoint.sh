#!/bin/sh

chown -R worker:worker /home/worker/dl
chown -R worker:worker /home/worker/output
chown -R worker:worker /home/worker/.cargo/registry
exec runuser -u worker "$@"
