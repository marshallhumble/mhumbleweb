# --------------------------------------------------
# Stage 1: Build Go binary securely with Wolfi base
# --------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base AS build

RUN apk add wolfictl

RUN wolfictl install go@1.25.1 tzdata binutils curl || apk add --no-cache go tzdata binutils curl

# Install cloudflared
RUN curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 \
    -o /usr/local/bin/cloudflared && chmod +x /usr/local/bin/cloudflared

ENV CGO_ENABLED=0
WORKDIR /src

COPY . .

RUN go mod download
RUN go build -ldflags "-s -w" -o web ./cmd/web && strip web

# --------------------------------------------------
# Stage 2: Runtime — secure and small
# --------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base AS runtime

RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

WORKDIR /var/www/html

# Copy just your Go binary and data
COPY --from=build /src/web ./web
COPY --from=build /src/internal/models/json/data.json ./internal/models/json/data.json

RUN chmod +x ./web && \
    chown -R appuser:appgroup /var/www/html

USER appuser
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Simple - just run your Go app
CMD ["./web", "-addr=0.0.0.0:8080"]