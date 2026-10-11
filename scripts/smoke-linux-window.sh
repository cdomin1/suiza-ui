#!/bin/sh
# Run inside xvfb-run. Confirm a real application window rather than just a process.
set -eu
log=$(mktemp)
"$@" > "$log" 2>&1 &
pid=$!
cleanup() {
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    cat "$log"
    rm -f "$log"
}
trap cleanup EXIT HUP INT TERM
attempt=0
while [ "$attempt" -lt 30 ]; do
    if ! kill -0 "$pid" 2>/dev/null; then
        printf 'Application exited before opening a window.\n' >&2
        exit 1
    fi
    if xwininfo -root -tree | grep -q 'SUIZA Canvas'; then
        printf 'SUIZA Canvas window opened successfully.\n'
        exit 0
    fi
    attempt=$((attempt + 1))
    sleep 1
done
printf 'No SUIZA Canvas window appeared within 30 seconds.\n' >&2
exit 1
