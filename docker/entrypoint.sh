#!/bin/bash
set -e

echo "=== Doghook Docker Entrypoint ==="

echo "Running database migrations..."
doghook doginals database migrate --config-path /etc/doghook.toml

echo "Starting doghook service..."

# Forward SIGTERM/SIGINT to the Rust process so tokio can drain gracefully.
_term() {
    echo "[entrypoint] Received SIGTERM — forwarding to doghook (pid $child)..."
    kill -TERM "$child" 2>/dev/null
    wait "$child"
}
trap _term SIGTERM SIGINT

doghook doginals service start --config-path /etc/doghook.toml &
child=$!
wait "$child"
