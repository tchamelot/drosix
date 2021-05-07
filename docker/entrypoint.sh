#!/bin/sh

chown -R worker:worker /home/worker/dl
chown -R worker:worker /home/worker/output
exec runuser -u worker "$@"
