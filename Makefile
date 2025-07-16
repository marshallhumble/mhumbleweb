include .env
export

# -------------------------
# Config
# -------------------------
GITHUB_USER=marshallhumble
REPO=mhumbleweb
IMAGE_NAME=ghcr.io/$(GITHUB_USER)/$(REPO)
BUILD_DATE := $(shell date +%Y%m%d)
IMAGE_TAG=$(BUILD_DATE)
FULL_IMAGE=$(IMAGE_NAME):$(IMAGE_TAG)

# Add version info for better tracking
GIT_COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_VERSION := $(BUILD_DATE)-$(GIT_COMMIT)

# -------------------------
# Help target
# -------------------------
.PHONY: help
help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# -------------------------
# Targets
# -------------------------

.PHONY: setup-tools
setup-tools: ## Install required tools (cosign, syft, grype, etc.)
	@echo "Installing tools: cosign, syft, trivy, wolfictl, buildx..."
	command -v cosign >/dev/null || brew install sigstore/tap/cosign
	command -v syft >/dev/null || brew install syft
	command -v trivy >/dev/null || brew install trivy
	command -v docker-buildx >/dev/null || docker buildx create --use
	command -v wolfictl >/dev/null || ( \
		git clone https://github.com/wolfi-dev/wolfictl.git /tmp/wolfictl && \
		cd /tmp/wolfictl && go install && \
		echo "wolfictl installed to \`go env GOPATH\`/bin" \
	)

.PHONY: login-ghcr
login-ghcr: ## Login to GitHub Container Registry
	@if [ -z "$(GHCR_TOKEN)" ]; then \
		echo "❌ GHCR_TOKEN not set. Please set it in .env or your shell environment."; \
		exit 1; \
	fi
	echo "$(GHCR_TOKEN)" | docker login ghcr.io -u $(GITHUB_USER) --password-stdin

.PHONY: update-base
update-base: ## Update wolfi base image in Dockerfile
	bash scripts/update-wolfi-base.sh Dockerfile

.PHONY: update-base-check
update-base-check: ## Check for newer wolfi-base version
	@echo "Checking for newer wolfi-base version..."
	bash scripts/update-wolfi-base.sh Dockerfile || echo "No update available."

.PHONY: build
build: ## Build and push Docker image
	@echo "Building image: $(FULL_IMAGE)"
	docker buildx build --platform linux/amd64 --tag $(FULL_IMAGE) --load .
	docker tag $(FULL_IMAGE) $(IMAGE_NAME):latest
	@echo "Pushing images..."
	docker push $(FULL_IMAGE)
	docker push $(IMAGE_NAME):latest

.PHONY: build-local
build-local: ## Build image locally without pushing
	@echo "Building image locally: $(FULL_IMAGE)"
	docker buildx build --platform linux/amd64 --tag $(FULL_IMAGE) --load .
	docker tag $(FULL_IMAGE) $(IMAGE_NAME):latest

.PHONY: sign
sign: ## Sign image with Cosign
	@echo "Signing image with Cosign..."
	COSIGN_EXPERIMENTAL=1 cosign sign --yes $(FULL_IMAGE)

.PHONY: attach-sbom
attach-sbom: ## Generate and attach SBOM
	@echo "Generating and attaching SBOM..."
	syft $(FULL_IMAGE) -o spdx-json > sbom.json
	cosign attach sbom --sbom sbom.json $(FULL_IMAGE)

.PHONY: attest
attest: ## Generate and attach SLSA provenance attestation
	@echo "Generating and attaching SLSA provenance attestation..."
	COSIGN_EXPERIMENTAL=1 cosign attest --yes \
		--predicate sbom.json \
		--type https://slsa.dev/provenance/v0.2 \
		$(FULL_IMAGE)

.PHONY: scan-cves
scan-cves: ## Scan image for CVEs with Trivy
	@echo "Scanning image for CVEs with Trivy..."
	trivy image $(FULL_IMAGE) || true

.PHONY: verify
verify: ## Verify image signature
	@echo "Verifying image signature..."
	COSIGN_EXPERIMENTAL=1 cosign verify $(FULL_IMAGE)

.PHONY: fly-deploy
fly-deploy: ## Deploy to Fly.io using latest image
	fly deploy --local-only --image $(IMAGE_NAME):latest

.PHONY: fly-deploy-tagged
fly-deploy-tagged: ## Deploy specific tagged image to Fly.io
	fly deploy --local-only --image $(FULL_IMAGE)

# -------------------------
# Combined workflows
# -------------------------

.PHONY: dev-build
dev-build: build-local scan-cves ## Quick development build with CVE scan

.PHONY: security-pipeline
security-pipeline: build sign attach-sbom attest verify scan-cves ## Complete security pipeline

.PHONY: full-pipeline
full-pipeline: setup-tools login-ghcr build sign attach-sbom attest verify scan-cves fly-deploy ## Complete CI/CD pipeline

# -------------------------
# Development helpers
# -------------------------

.PHONY: test-local
test-local: ## Test the application locally
	go test ./...

.PHONY: run-local
run-local: ## Run the application locally
	go run ./cmd/web

.PHONY: clean
clean: ## Clean up generated files
	rm -f sbom.json sbom.html
	docker image prune -f

.PHONY: show-config
show-config: ## Show current configuration
	@echo "Configuration:"
	@echo "  GITHUB_USER: $(GITHUB_USER)"
	@echo "  REPO: $(REPO)"
	@echo "  IMAGE_NAME: $(IMAGE_NAME)"
	@echo "  BUILD_DATE: $(BUILD_DATE)"
	@echo "  GIT_COMMIT: $(GIT_COMMIT)"
	@echo "  BUILD_VERSION: $(BUILD_VERSION)"
	@echo "  FULL_IMAGE: $(FULL_IMAGE)"