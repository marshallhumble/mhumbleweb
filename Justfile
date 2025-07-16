# Load environment variables
set dotenv-load

# Variables
github_user := "marshallhumble"
repo := "mhumbleweb"
image_name := "ghcr.io/" + github_user + "/" + repo
build_date := `date +%Y%m%d`
git_commit := `git rev-parse --short HEAD 2>/dev/null || echo "unknown"`
full_image := image_name + ":" + build_date

# Show available recipes
default:
    @just --list

# Install required tools
setup-tools:
    @echo "Installing tools: cosign, syft, trivy, wolfictl, buildx..."
    command -v cosign >/dev/null || brew install sigstore/tap/cosign
    command -v syft >/dev/null || brew install syft
    command -v trivy >/dev/null || brew install trivy
    command -v docker-buildx >/dev/null || docker buildx create --use

# Login to GitHub Container Registry
login-ghcr:
    #!/usr/bin/env bash
    if [ -z "$GHCR_TOKEN" ]; then
        echo "❌ GHCR_TOKEN not set. Please set it in .env"
        exit 1
    fi
    echo "$GHCR_TOKEN" | docker login ghcr.io -u {{github_user}} --password-stdin

# Build and push Docker image
build:
    @echo "Building: {{full_image}}"
    docker buildx build --platform linux/amd64 --tag {{full_image}} --load .
    docker tag {{full_image}} {{image_name}}:latest
    docker push {{full_image}}
    docker push {{image_name}}:latest

# Build locally without pushing
build-local:
    docker buildx build --platform linux/amd64 --tag {{full_image}} --load .
    docker tag {{full_image}} {{image_name}}:latest

# Sign image with Cosign
sign:
    COSIGN_EXPERIMENTAL=1 cosign sign --yes {{full_image}}

# Generate and attach SBOM
attach-sbom:
    syft {{full_image}} -o spdx-json > sbom.json
    cosign attach sbom --sbom sbom.json {{full_image}}

# Generate SLSA provenance attestation
attest:
    COSIGN_EXPERIMENTAL=1 cosign attest --yes --predicate sbom.json --type https://slsa.dev/provenance/v0.2 {{full_image}}

# Scan for CVEs
scan-cves:
    trivy image {{full_image}} || true

# Verify image signature
verify:
    COSIGN_EXPERIMENTAL=1 cosign verify {{full_image}}

# Deploy to Fly.io
fly-deploy:
    fly deploy --local-only --image {{image_name}}:latest

# Run tests locally
test:
    go test ./...

# Run app locally
run:
    go run ./cmd/web

# Clean up
clean:
    rm -f sbom.json sbom.html
    docker image prune -f

# Show current config
show-config:
    @echo "GitHub User: {{github_user}}"
    @echo "Repo: {{repo}}"
    @echo "Image: {{image_name}}"
    @echo "Full Image: {{full_image}}"
    @echo "Git Commit: {{git_commit}}"

# Complete pipeline
full-pipeline: setup-tools login-ghcr build sign attach-sbom attest verify scan-cves fly-deploy

# Development build
dev-build: build-local scan-cves

# Security pipeline only
security-pipeline: build sign attach-sbom attest verify scan-cves