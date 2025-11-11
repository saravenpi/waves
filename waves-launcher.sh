#!/bin/bash

WAVES_BIN="$(dirname "$0")/waves"

if [ $# -eq 0 ]; then
    exec "$WAVES_BIN"
else
    exec "$WAVES_BIN" "$@"
fi
