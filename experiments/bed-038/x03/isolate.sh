#!/bin/sh
set -eu
mount -o remount,ro /proc
exec setpriv --reuid=65534 --regid=65534 --clear-groups --bounding-set=-all --inh-caps=-all --ambient-caps=-all --no-new-privs /usr/bin/python3 -I -B /src/experiments/bed-038/x03/launch.py "$@"
