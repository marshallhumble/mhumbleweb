ARG GO_VERSION=1.22.6

# First stage: build the executable.
FROM golang:${GO_VERSION}-alpine AS build
ENV CGO_ENABLED=0
WORKDIR /src
COPY . .
RUN go mod download
RUN go build -ldflags "-s -w" -o web ./cmd/web

FROM alpine:edge

WORKDIR /app

COPY --from=build /src/web .
COPY --from=build ./tls/cert.pem /app/tls/cert.pem
COPY --from=build ./tls/key.pem /app/tls/key.pem


EXPOSE 443
ENTRYPOINT ["/app/web"]

