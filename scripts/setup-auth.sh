#!/usr/bin/env bash
#
# setup-auth.sh - Interactive script to create API authentication credentials
#
# This script helps you:
# 1. Create a credentials.json file with username/password entries
# 2. Generate secure password hashes using PBKDF2-HMAC-SHA256
# 3. Optionally update spec-ai.config.toml to enable auth
#
# Usage: ./scripts/setup-auth.sh [--output PATH]
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default paths
DEFAULT_CREDENTIALS_FILE="$HOME/.spec-ai/credentials.json"
DEFAULT_CONFIG_FILE="spec-ai.config.toml"

# Parse arguments
OUTPUT_FILE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --output|-o)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [--output PATH]"
            echo ""
            echo "Options:"
            echo "  --output, -o PATH    Output path for credentials file (default: ~/.spec-ai/credentials.json)"
            echo "  --help, -h           Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Use default if not specified
if [[ -z "$OUTPUT_FILE" ]]; then
    OUTPUT_FILE="$DEFAULT_CREDENTIALS_FILE"
fi

echo -e "${BLUE}=== spec-ai Authentication Setup ===${NC}"
echo ""

# Check if spec-ai binary is available for hashing
SPEC_AI_BIN=""
if command -v spec-ai &> /dev/null; then
    SPEC_AI_BIN="spec-ai"
elif [[ -f "./target/release/spec-ai" ]]; then
    SPEC_AI_BIN="./target/release/spec-ai"
elif [[ -f "./target/debug/spec-ai" ]]; then
    SPEC_AI_BIN="./target/debug/spec-ai"
fi

# Function to hash password using ring (same algorithm as the server)
hash_password() {
    local password="$1"

    # If we have the spec-ai binary with a hash subcommand, use it
    # Otherwise, we'll need to use the server's /auth/hash endpoint
    # For now, we'll generate using openssl as a fallback with compatible format

    # Generate 16-byte salt
    local salt=$(openssl rand -hex 16)

    # Derive key using PBKDF2-HMAC-SHA256 with 100000 iterations
    # Output is 32 bytes (256 bits)
    local derived=$(echo -n "$password" | openssl pkcs5 -pbkdf2 -iter 100000 -md sha256 -S "$salt" -hex 2>/dev/null | tail -1)

    # Combine salt + derived key and base64 encode (URL-safe, no padding)
    local combined=$(echo -n "${salt}${derived}" | xxd -r -p | base64 | tr '+/' '-_' | tr -d '=')

    echo "$combined"
}

# Alternative: use Python if available (more reliable PBKDF2)
hash_password_python() {
    local password="$1"
    python3 << EOF
import hashlib
import os
import base64

password = "$password".encode()
salt = os.urandom(16)
derived = hashlib.pbkdf2_hmac('sha256', password, salt, 100000, dklen=32)
combined = salt + derived
# URL-safe base64 without padding
encoded = base64.urlsafe_b64encode(combined).rstrip(b'=').decode()
print(encoded)
EOF
}

# Check which hashing method to use
HASH_METHOD=""
if command -v python3 &> /dev/null; then
    # Verify Python has hashlib
    if python3 -c "import hashlib" 2>/dev/null; then
        HASH_METHOD="python"
    fi
fi

if [[ -z "$HASH_METHOD" ]]; then
    echo -e "${YELLOW}Warning: Python3 not found. Password hashing may not be compatible.${NC}"
    echo -e "${YELLOW}Consider running the server and using POST /auth/hash endpoint instead.${NC}"
    echo ""
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
    HASH_METHOD="openssl"
fi

# Create output directory if needed
OUTPUT_DIR=$(dirname "$OUTPUT_FILE")
if [[ ! -d "$OUTPUT_DIR" ]]; then
    echo -e "${YELLOW}Creating directory: $OUTPUT_DIR${NC}"
    mkdir -p "$OUTPUT_DIR"
fi

# Check if credentials file already exists
if [[ -f "$OUTPUT_FILE" ]]; then
    echo -e "${YELLOW}Credentials file already exists: $OUTPUT_FILE${NC}"
    read -p "Overwrite? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 0
    fi
fi

# Collect user credentials
echo -e "${GREEN}Enter user credentials (press Enter with empty username to finish)${NC}"
echo ""

CREDENTIALS="["
FIRST=true
USER_COUNT=0

while true; do
    read -p "Username: " username

    if [[ -z "$username" ]]; then
        if [[ $USER_COUNT -eq 0 ]]; then
            echo -e "${RED}At least one user is required.${NC}"
            continue
        fi
        break
    fi

    # Read password securely
    read -s -p "Password: " password
    echo
    read -s -p "Confirm password: " password_confirm
    echo

    if [[ "$password" != "$password_confirm" ]]; then
        echo -e "${RED}Passwords do not match. Try again.${NC}"
        continue
    fi

    if [[ ${#password} -lt 8 ]]; then
        echo -e "${YELLOW}Warning: Password is less than 8 characters.${NC}"
        read -p "Continue anyway? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            continue
        fi
    fi

    # Hash the password
    echo -e "${BLUE}Hashing password...${NC}"
    if [[ "$HASH_METHOD" == "python" ]]; then
        password_hash=$(hash_password_python "$password")
    else
        password_hash=$(hash_password "$password")
    fi

    # Add to JSON array
    if [[ "$FIRST" == "true" ]]; then
        FIRST=false
    else
        CREDENTIALS+=","
    fi

    CREDENTIALS+=$'\n  {"username": "'"$username"'", "password_hash": "'"$password_hash"'"}'
    USER_COUNT=$((USER_COUNT + 1))

    echo -e "${GREEN}Added user: $username${NC}"
    echo ""
done

CREDENTIALS+=$'\n]'

# Write credentials file
echo "$CREDENTIALS" > "$OUTPUT_FILE"
chmod 600 "$OUTPUT_FILE"  # Restrict permissions

echo ""
echo -e "${GREEN}Created credentials file: $OUTPUT_FILE${NC}"
echo -e "${GREEN}Added $USER_COUNT user(s)${NC}"
echo ""

# Offer to update config file
if [[ -f "$DEFAULT_CONFIG_FILE" ]]; then
    echo -e "${BLUE}Would you like to update $DEFAULT_CONFIG_FILE to enable authentication?${NC}"
    read -p "Update config? [y/N] " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Check if [auth] section exists
        if grep -q '^\[auth\]' "$DEFAULT_CONFIG_FILE"; then
            # Update existing section
            # Use sed to update enabled and credentials_file
            if [[ "$OSTYPE" == "darwin"* ]]; then
                # macOS sed requires different syntax
                sed -i '' 's/^enabled = false/enabled = true/' "$DEFAULT_CONFIG_FILE"
                sed -i '' "s|^# credentials_file = .*|credentials_file = \"$OUTPUT_FILE\"|" "$DEFAULT_CONFIG_FILE"
            else
                sed -i 's/^enabled = false/enabled = true/' "$DEFAULT_CONFIG_FILE"
                sed -i "s|^# credentials_file = .*|credentials_file = \"$OUTPUT_FILE\"|" "$DEFAULT_CONFIG_FILE"
            fi
            echo -e "${GREEN}Updated $DEFAULT_CONFIG_FILE${NC}"
        else
            echo -e "${YELLOW}[auth] section not found in config. Please add manually:${NC}"
            echo ""
            echo "[auth]"
            echo "enabled = true"
            echo "credentials_file = \"$OUTPUT_FILE\""
        fi
    fi
fi

echo ""
echo -e "${GREEN}=== Setup Complete ===${NC}"
echo ""
echo "To use authentication:"
echo "1. Start the server: spec-ai server"
echo "2. Get a token: curl -X POST http://localhost:3000/auth/token \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"username\": \"YOUR_USER\", \"password\": \"YOUR_PASSWORD\"}'"
echo "3. Use the token: curl http://localhost:3000/agents \\"
echo "     -H 'Authorization: Bearer YOUR_TOKEN'"
echo ""
