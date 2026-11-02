# --------------------------------------------------
# Stage 1: Build
# --------------------------------------------------
FROM rust:alpine@sha256:7e9e2d068a107a2adafd19930011178d14a78326053cd4c3a9f7ad3fd049557d AS build

RUN apk add --no-cache musl-dev

WORKDIR /src
COPY . .

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release --target x86_64-unknown-linux-musl

# --------------------------------------------------
# Stage 2: Runtime
# --------------------------------------------------
FROM scratch AS runtime

WORKDIR /app

COPY --from=build --chown=1001:1001 /src/target/x86_64-unknown-linux-musl/release/mhumbleweb ./mhumbleweb
COPY --from=build --chown=1001:1001 /src/templates ./templates
COPY --from=build --chown=1001:1001 /src/static ./static
COPY --from=build --chown=1001:1001 /src/internal/models/json/data.json ./internal/models/json/data.json

USER 1001:1001
EXPOSE 3000

CMD ["./mhumbleweb"]