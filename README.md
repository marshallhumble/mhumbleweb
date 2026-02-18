# mhumble.io

Personal site built with Rust and Axum. Migrated from Go in 2026.

## Stack

- **Axum 0.8** — async web framework
- **Tera** — template engine
- **tower-http** — static file serving and security headers
- **Tokio** — async runtime
- **Alpine Linux** — minimal runtime container

## Project Structure

```
mhumbleweb/
├── src/
│   ├── main.rs          # App entry point, router, AppState
│   ├── handlers.rs      # Route handlers
│   ├── middleware.rs    # Security headers
│   ├── models.rs        # Post struct, JSON loader
│   └── view_models.rs   # Template view models
├── templates/
│   ├── base.html        # Shared layout
│   ├── index.html       # Homepage
│   ├── articles.html    # Article list with topic filter
│   ├── article.html     # Single article view
│   ├── about.html       # About page
│   └── articles/        # Article content (HTML)
├── static/
│   ├── css/main.css     # Terminal aesthetic styles
│   ├── js/prism.js      # Syntax highlighting
│   └── images/          # Static images
├── internal/models/json/
│   └── data.json        # Article metadata
├── Dockerfile           # Two-stage build (rust:alpine → alpine)
├── fly.toml             # Fly.io deployment config
└── Justfile             # Build, sign, scan, deploy recipes
```

## Local Development

```bash
# Run in dev mode
just run

# Run with release optimizations
just run-release

# Check and lint
just check
```

## Security

Security headers are applied globally via tower-http middleware:

- Content-Security-Policy
- Strict-Transport-Security (HSTS)
- X-Content-Type-Options
- X-Frame-Options
- Referrer-Policy
- Permissions-Policy

## Building and Deploying

```bash
# Install required tools
just setup-tools

# Login to GHCR
just login-ghcr

# Full pipeline: build, sign, SBOM, attest, scan, deploy
just full-pipeline

# Or run steps individually
just build
just sign
just attach-sbom
just scan-cves
just fly-deploy
```

Requires a `.env` file with:

```
GHCR_TOKEN=your_github_pat
```

## Image Signing and Supply Chain

Images are signed with Cosign, SBOM generated with Syft in SPDX format,
SLSA provenance attestation attached, and CVE scanned with Trivy before deploy.

```bash
# Verify a deployed image
just verify
```

## Deployment

Hosted on Fly.io in the `dfw` region. TLS is terminated at the Fly.io edge.
The container runs plain HTTP internally on port 3000.

```bash
fly deploy --local-only --image ghcr.io/marshallhumble/mhumbleweb:latest
```