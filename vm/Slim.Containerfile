# Slim runtime image for spec-ai
# Fetches the latest release binary from GitHub
#
# Build: podman build -t spec-ai:latest .
#    or: docker build -t spec-ai:latest .

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    # SSL/TLS runtime \
    ca-certificates \
    libssl3 \
    # Audio runtime (for audio features) \
    libasound2 \
    # OCR runtime (for document extraction) \
    tesseract-ocr \
    tesseract-ocr-eng \
    # For downloading release \
    curl \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash spec-ai

WORKDIR /app

# Fetch and install the latest release binary
RUN set -eux; \
    ARCH=$(uname -m); \
    case "$ARCH" in \
        x86_64) TRIPLE="x86_64-unknown-linux-gnu" ;; \
        aarch64) TRIPLE="aarch64-unknown-linux-gnu" ;; \
        *) echo "Unsupported architecture: $ARCH" && exit 1 ;; \
    esac; \
    # Get latest release tag \
    LATEST_TAG=$(curl -fsSL https://api.github.com/repos/geoffsee/spec-ai/releases/latest | jq -r '.tag_name'); \
    echo "Fetching spec-ai ${LATEST_TAG} for ${TRIPLE}"; \
    # Download and extract \
    curl -fsSL "https://github.com/geoffsee/spec-ai/releases/download/${LATEST_TAG}/spec-ai-${LATEST_TAG}-${TRIPLE}.tar.gz" \
        | tar -xzf - -C /usr/local/bin; \
    chmod +x /usr/local/bin/spec-ai; \
    # Verify installation \
    /usr/local/bin/spec-ai --version || true

# Clean up download tools (optional, keeps image smaller)
RUN apt-get purge -y curl jq && apt-get autoremove -y && rm -rf /var/lib/apt/lists/*

# Switch to non-root user
USER spec-ai
WORKDIR /home/spec-ai

# Default data directory
ENV SPEC_AI_DATA_DIR=/home/spec-ai/.spec-ai

ENTRYPOINT ["spec-ai"]
CMD ["--help"]
