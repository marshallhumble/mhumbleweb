ARG GO_VERSION=1.24.2

# First stage: build the executable.
FROM golang:${GO_VERSION}-alpine AS build
RUN apk update && apk upgrade --no-cache


RUN apk add --no-cache tzdata

ENV CGO_ENABLED=0

WORKDIR /src

COPY . .
RUN go mod download
RUN go build -ldflags "-s -w" -o web ./cmd/web

FROM scratch

COPY --from=build /usr/share/zoneinfo /usr/share/zoneinfo
ENV TZ=America/Chicago


#set workdir
WORKDIR /var/www/html

COPY --from=build /src/web /var/www/html/web
COPY --from=build /src/tls/cert.pem /var/www/html/tls/cert.pem
COPY --from=build /src/tls/key.pem /var/www/html/tls/key.pem
COPY --from=build /src/internal/models/json/data.json /var/www/html/internal/models/json/data.json

EXPOSE 443
ENTRYPOINT ["/var/www/html/web"]

