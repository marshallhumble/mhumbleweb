#!/bin/bash
set -euo pipefail

# Check if sops and yq are installed
command -v sops >/dev/null 2>&1 || { echo "❌ sops is not installed"; exit 1; }
command -v yq >/dev/null 2>&1 || { echo "❌ yq is not installed"; exit 1; }

# Paths
KEY_FILE="age.key"
ENCRYPTED_FILE="cert.enc.yaml"
OUTPUT_CERT="/etc/cloudflared/cert.pem"

# Validate presence of encrypted file
if [[ ! -f "$ENCRYPTED_FILE" ]]; then
  echo "❌ Encrypted cert file not found: $ENCRYPTED_FILE"
  exit 1
fi

# Validate presence of key file
if [[ ! -f "$KEY_FILE" ]]; then
  echo "❌ Age key file not found: $KEY_FILE"
  exit 1
fi

# Export key for sops
export SOPS_AGE_KEY_FILE=$KEY_FILE

# Decrypt
mkdir -p $(dirname "$OUTPUT_CERT")
echo "🔐 Decrypting cert.pem..."
sops -d "$ENCRYPTED_FILE" | yq -r ."cert.pem" > "$OUTPUT_CERT"
echo "✅ Decrypted to $OUTPUT_CERT"
