ARG GO_VERSION=1.23.3

# First stage: build the executable.
FROM golang:${GO_VERSION}-alpine AS build
ENV CGO_ENABLED=0
ENV TZ=CST
WORKDIR /src
COPY . .
RUN go mod download
RUN go build -ldflags "-s -w" -o web ./cmd/web

FROM scratch

ENV TZ=CST
WORKDIR /app

COPY --from=build /src/web .
COPY --from=build /src/tls/cert.pem /app/tls/cert.pem
COPY --from=build /src/tls/key.pem /app/tls/key.pem
COPY --from=build /src/internal/models/json/data.json /app/internal/models/json/data.json

EXPOSE 443
ENTRYPOINT ["/app/web"]

