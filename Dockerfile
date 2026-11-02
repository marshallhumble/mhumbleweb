# --------------------------------------------------
# Stage 1: Build
# --------------------------------------------------
FROM rust:alpine@sha256:be2656061b536c1883c4034790e21d0a09c5aaaf51968b89368a3c0371ea2c9a AS build

RUN apk add --no-cache musl-dev

WORKDIR /src
COPY . .

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release --target x86_64-unknown-linux-musl

# --------------------------------------------------
# Stage 2: Runtime
# --------------------------------------------------
FROM alpine:3.24.3@sha256:5aec467fb81238bae9441eea2edb000655d7044f AS runtime

RUN apk update && apk upgrade --no-cache

RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

WORKDIR /app

COPY --from=build /src/target/x86_64-unknown-linux-musl/release/mhumbleweb ./mhumbleweb
COPY --from=build /src/templates ./templates
COPY --from=build /src/static ./static
COPY --from=build /src/internal/models/json/data.json ./internal/models/json/data.json

RUN chmod +x ./mhumbleweb && \
    chown -R appuser:appgroup /app

USER appuser
EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget -qO- http://localhost:3000/health || exit 1

CMD ["./mhumbleweb"]