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

# -------------------------
# Targets
# -------------------------

setup-tools:
	@echo "Installing tools: cosign, syft, grype, wolfictl, buildx..."
	command -v cosign >/dev/null || brew install sigstore/tap/cosign
	command -v syft >/dev/null || brew install syft
	command -v grype >/dev/null || brew install grype
	command -v docker-buildx >/dev/null || docker buildx create --use
	command -v wolfictl >/dev/null || ( \
	  git clone https://github.com/wolfi-dev/wolfictl.git /tmp/wolfictl && \
	  cd /tmp/wolfictl && go install && \
	  echo "wolfictl installed to \`go env GOPATH\`/bin" \
	)

login-ghcr:
	@if [ -z "$(GHCR_TOKEN)" ]; then \
	  echo "❌ GHCR_TOKEN not set. Please set it in .env or your shell environment."; \
	  exit 1; \
	fi
	echo "$(GHCR_TOKEN)" | docker login ghcr.io -u $(GITHUB_USER) --password-stdin

update-base:
	bash scripts/update-wolfi-base.sh Dockerfile

update-base-check:
	@echo "Checking for newer wolfi-base version..."
	bash scripts/update-wolfi-base.sh Dockerfile || echo "No update available."

build:
	docker buildx build --platform linux/amd64 --tag $(FULL_IMAGE) --load .
	docker tag $(FULL_IMAGE) $(IMAGE_NAME):latest
	docker push $(FULL_IMAGE)
	docker push $(IMAGE_NAME):latest

sign:
	@echo "Signing image with Cosign..."
	COSIGN_EXPERIMENTAL=1 cosign sign --yes $(FULL_IMAGE)

attach-sbom:
	@echo "Generating and attaching SBOM..."
	syft $(FULL_IMAGE) -o spdx-json > sbom.json
	cosign attach sbom --sbom sbom.json $(FULL_IMAGE)

attest:
	@echo "Generating and attaching SLSA provenance attestation..."
	COSIGN_EXPERIMENTAL=1 cosign attest --yes \
		--predicate sbom.json \
		--type https://slsa.dev/provenance/v0.2 \
		$(FULL_IMAGE)

scan-cves:
	@echo "Scanning image for CVEs with Grype..."
	grype $(FULL_IMAGE) || true

verify:
	COSIGN_EXPERIMENTAL=1 cosign verify $(FULL_IMAGE)

full-pipeline: setup-tools login-ghcr build sign attach-sbom attest verify scan-cves

clean:
	rm -f sbom.json sbom.html
