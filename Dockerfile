# --------------------------------------------------
# Stage 1: Build Go binary securely with Wolfi base
# --------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base AS build

RUN apk add wolfictl

RUN wolfictl install go@1.24.5 tzdata binutils curl || apk add --no-cache go tzdata binutils curl

# Install cloudflared
RUN curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 \
    -o /usr/local/bin/cloudflared && chmod +x /usr/local/bin/cloudflared

ENV CGO_ENABLED=0
WORKDIR /src

COPY . .

RUN go mod download
RUN go build -ldflags "-s -w" -o web ./cmd/web && strip web

# --------------------------------------------------
# Stage 2: Runtime — secure and small with supervisor
# --------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base AS runtime

# Install supervisor for process management
RUN apk add --no-cache supervisor

# Create a non-root user
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

# Create necessary directories
# Create necessary directories with proper permissions
RUN mkdir -p /var/www/html \
             /var/log/supervisor \
             /var/run \
             /etc/supervisor/conf.d && \
    chown -R appuser:appgroup /var/www/html /var/log/supervisor && \
    chmod 755 /var/run

WORKDIR /var/www/html

# Copy binaries from build stage
COPY --from=build /usr/local/bin/cloudflared /usr/local/bin/cloudflared
COPY --from=build /src/web ./web
COPY --from=build /src/internal/models/json/data.json ./internal/models/json/data.json

# Create supervisor configuration
RUN cat > /etc/supervisor/conf.d/supervisord.conf << 'EOF'
[supervisord]
nodaemon=true
user=root
logfile=/var/log/supervisor/supervisord.log
pidfile=/var/run/supervisord.pid

[program:webapp]
command=/var/www/html/web -addr=0.0.0.0:80
directory=/var/www/html
user=appuser
autostart=true
autorestart=true
stderr_logfile=/var/log/supervisor/webapp.log
stdout_logfile=/var/log/supervisor/webapp.log

[program:cloudflared]
command=/usr/local/bin/cloudflared tunnel --no-autoupdate run --token %(ENV_TUNNEL_TOKEN)s
user=appuser
autostart=true
autorestart=true
stderr_logfile=/var/log/supervisor/cloudflared.log
stdout_logfile=/var/log/supervisor/cloudflared.log
EOF

# Set proper permissions
RUN chmod +x ./web && \
    chown -R appuser:appgroup /var/www/html

# Expose port 80 (for internal HTTP)
EXPOSE 80

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:80/health || exit 1

# Start supervisor to manage both processes
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
