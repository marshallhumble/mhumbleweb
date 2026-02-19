# --------------------------------------------------
# Stage 1: Build
# --------------------------------------------------
FROM rust:alpine@sha256:4fec02de605563c297c78a31064c8335bc004fa2b0bf406b1b99441da64e2d2d AS build

RUN apk add --no-cache musl-dev

WORKDIR /src
COPY . .

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release --target x86_64-unknown-linux-musl

# --------------------------------------------------
# Stage 2: Runtime
# --------------------------------------------------
FROM alpine:3.21@sha256:c3f8e73fdb79deaebaa2037150150191b9dcbfba68b4a46d70103204c53f4709 AS runtime

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