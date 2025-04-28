#!/bin/sh
set -euo pipefail

# Start the Go web server in the background
/var/www/html/web &
WEB_PID=$!

echo "🟢 Go server started with PID $WEB_PID"

# Start cloudflared in the foreground
exec /usr/local/bin/cloudflared tunnel --no-autoupdate run --token ${TUNNEL_TOKEN}
