
# 🧠 Go Personal Site with Secure Docker

This project hosts your personal Go-powered site securely using:

- 🐳 **Docker** with `wolfi-base` for minimal and secure builds
- 🧾 **Cosign** + **SBOMs** + **Grype** for image signing and vulnerability scanning
- 🚀 **GitHub Actions** to automate build, push, sign, and verify

---

## 🧱 Folder Structure

```bash
go-personal-site/
├── cmd/                   # Go app entrypoint
│   └── web/
├── internal/models/json/  # Data model or content
├── cert.yaml             # SOPS-encrypted cert source
├── cert.enc.yaml         # Encrypted Cloudflare cert
├── age.key               # Local age private key (gitignored)
├── Dockerfile
├── Makefile
├── .gitignore
├── go.mod / go.sum
```

---

## 🚀 Quick Start

### 🔧 Local Setup
```bash
brew install sops age
brew install cosign syft grype
```

### 🔐 Encode Cloudflare Cert for GitHub
```bash
base64 -i cloudflared/cert.pem > cert.pem.b64
```
Add `cert.pem.b64` to GitHub Secrets as `CF_CERT_B64`.

---

## 🛠 Local Testing

Run this to build and test locally:
```bash
bash scripts/test-local.sh
```

✅ This will:
- Build the image as `local-go-site:dev`
- Mount your `cloudflared/` directory
- Run the app on port 443

---

## 🧪 Decryption (Local or CI)

Recreate `cloudflared/cert.pem` from SOPS-encrypted cert:
```bash
bash scripts/decrypt-cert.sh
```

---

## 🏗 Secure CI/CD Pipeline (GitHub Actions)

The `Makefile` automates your full production build pipeline:

```bash
make full-pipeline
```

This will:
- 🔄 Update your `wolfi-base` image
- 🔧 Build the Docker image
- 🔐 Sign it with Cosign
- 📦 Generate and attach SBOM
- 🧾 Create and attach SLSA provenance
- 🛡️ Scan for CVEs using Grype
- ✅ Verify signatures

---

## 🔐 Secrets to Add in GitHub

| Secret Name     | Description                          |
|----------------|--------------------------------------|
| `GHCR_TOKEN`    | GitHub PAT with `write:packages`     |
| `CF_CERT_B64`   | base64-encoded cloudflared cert.pem  |
| `AGE_PRIVATE_KEY_B64` | base64 of your `age.key` file     |

---

## 📦 Deployment (Fly.io, etc.)

Use the signed image from `ghcr.io/<user>/<repo>:tag`
```bash
flyctl deploy --image ghcr.io/<user>/<repo>:tag
```

You can also run Cloudflare Tunnel independently:
```bash
cloudflared tunnel --config cloudflared/config.yml run
```

---

## 📋 .gitignore Notes

```gitignore
cloudflared/cert.pem
tls/cert.pem
tls/key.pem
cert.yaml
cert.pem.b64
age.key
sbom.json
```

---

## 📣 Need Help?
- [Cloudflare Tunnel Docs](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/)
- [Cosign](https://docs.sigstore.dev/cosign/overview/)
- [SOPS](https://github.com/mozilla/sops)
- [Fly.io](https://fly.io/docs/)

---

Stay secure. Stay reproducible. 🚀
