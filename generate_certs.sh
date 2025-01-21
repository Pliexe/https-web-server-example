#!/bin/bash
set -e

# Check if OpenSSL is installed
if ! command -v openssl >/dev/null 2>&1; then
    echo "Error: OpenSSL is not installed or not in PATH"
    exit 1
fi

echo "WARNING: This script will:"
echo " - Create a 'certs' folder in the current directory"
echo " - Generate SSL certificates using OpenSSL"
echo

# Prompt for confirmation
read -p "Do you want to proceed? (Y/N): " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
    echo "Operation cancelled by user."
    exit 1
fi

echo "Begin setup..."

# Get the directory of the current script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Create the "certs" folder if it doesn't exist
if [ ! -d "$SCRIPT_DIR/certs" ]; then
    echo "Folder 'certs' missing. Creating folder."
    mkdir "$SCRIPT_DIR/certs" || exit 1
fi

# Change to the certs directory
cd "$SCRIPT_DIR/certs" || exit 1

# Generate CA private key and certificate
echo "Generating CA private key and certificate..."
openssl genrsa -out ca.key 2048
openssl req -x509 -new -nodes -key ca.key -sha256 -days 3650 \
    -out ca.pem -subj "/CN=Local Development CA"

# Generate server private key
echo "Generating server private key..."
openssl genrsa -out localhost-key.pem 2048

# Create config file for certificate
cat > openssl.cnf << EOF
[req]
default_bits = 2048
prompt = no
default_md = sha256
distinguished_name = dn
req_extensions = req_ext

[dn]
C = US
ST = LocalState
L = LocalCity
O = LocalOrganization
OU = Development
CN = localhost

[req_ext]
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

# Generate certificate signing request
echo "Generating certificate signing request..."
openssl req -new -key localhost-key.pem -out localhost.csr \
    -config openssl.cnf

# Generate server certificate
echo "Generating server certificate..."
openssl x509 -req -in localhost.csr -CA ca.pem -CAkey ca.key \
    -CAcreateserial -out localhost.pem -days 365 \
    -sha256 -extensions req_ext -extfile openssl.cnf

# Clean up temporary files
rm localhost.csr openssl.cnf ca.key ca.srl

echo "Generation of certificates completed successfully."

# Make the certificate files readable only by the owner
chmod 600 localhost-key.pem localhost.pem ca.pem

echo "Note: You may need to manually trust ca.pem in your system/browser."