#!/bin/bash
set -euo pipefail

DOCKERFILE=${1:-"Dockerfile"}
IMAGE_NAME="wolfi-base"
FALLBACK_VERSION="latest"

# Use wolfictl if available
#if ! command -v wolfictl &>/dev/null; then
 # echo "Installing wolfictl..."
 # curl -sSfL https://github.com/wolfi-dev/wolfictl/releases/latest/download/wolfictl-darwin-arm64 -o /usr/local/bin/wolfictl
#  chmod +x /usr/local/bin/wolfictl
#fi

# Get the latest version
LATEST_VERSION=$(wolfictl ls versions chainguard/${IMAGE_NAME} 2>/dev/null | grep '^v' | sort -V | tail -n 1 || echo "$FALLBACK_VERSION")

echo "Updating Dockerfile to use wolfi-base:${LATEST_VERSION}"

# Replace the base image in Dockerfile (macOS compatible sed)
sed -i '' -E "s|FROM cgr.dev/chainguard/wolfi-base:[^[:space:]]*|FROM cgr.dev/chainguard/wolfi-base:${LATEST_VERSION}|" "$DOCKERFILE"
