# CI Base Image for spec-ai
# Pre-installs all system dependencies needed to build the project.
#
# Build: docker build -t spec-ai-ci-base:latest -f vm/CI.Dockerfile .

FROM rust:latest

# Install system dependencies
RUN apt-get update && apt-get install -y \
    # Build essentials
    pkg-config \
    cmake \
    build-essential \
    # SSL/TLS
    libssl-dev \
    # Network tools
    curl \
    wget \
    # Archive tools
    unzip \
    # Audio (for spider/media dependencies)
    libasound2-dev \
    # OCR support (for extractous)
    tesseract-ocr \
    tesseract-ocr-eng \
    # Java (required for extractous/tika-native)
    default-jdk \
    # Clang/LLVM (for bindgen)
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Set JAVA_HOME for extractous/tika builds
ENV JAVA_HOME=/usr/lib/jvm/default-java

WORKDIR /build

CMD ["rustc", "--version"]