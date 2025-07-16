# --------------------------------------------------
# Stage 1: Build Go binary securely with Wolfi base
# --------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base AS build

RUN apk add wolfictl

RUN wolfictl install go@1.24.5 tzdata binutils curl || apk add --no-cache go tzdata binutils curl

ENV CGO_ENABLED=0
WORKDIR /src

COPY . .

RUN go mod download
RUN go build -ldflags "-s -w" -o web ./cmd/web && strip web

# --------------------------------------------------
# Stage 2: Runtime — secure and small (wolfi-base)
# --------------------------------------------------
FROM cgr.dev/chainguard/wolfi-base AS runtime

WORKDIR /var/www/html

# Copy runtime binaries and assets
COPY --from=build /src/web ./web
COPY --from=build /src/tls/cert.pem ./tls/cert.pem
COPY --from=build /src/tls/key.pem ./tls/key.pem
COPY --from=build /src/internal/models/json/data.json ./internal/models/json/data.json

EXPOSE 443

# Run the Go application directly
CMD ["/var/www/html/web"]