ARG GO_VERSION=1.22.5

# First stage: build the executable.
FROM golang:${GO_VERSION}-alpine AS build

WORKDIR /src
COPY . .
ENV CGO_ENABLED=1
RUN go mod download
RUN go build -o web ./cmd/web

FROM alpine:edge

WORKDIR /app

COPY --from=build /src/web .
COPY ./tls/ .
COPY ./articles.sqlite .

EXPOSE 8080
ENTRYPOINT ["/app/web"]

