#!/bin/bash
set -e

# Publish spec-ai crates in dependency order
#
# Dependency graph:
#   spec-ai-config (no internal deps)
#   spec-ai-policy -> spec-ai-config
#   spec-ai-core   -> spec-ai-config, spec-ai-policy
#   spec-ai-api    -> spec-ai-core, spec-ai-config, spec-ai-policy
#   spec-ai        -> spec-ai-core, spec-ai-config, spec-ai-policy, spec-ai-api
#   spec-ai-cli    -> spec-ai

CRATES=(
    "spec-ai-config"
    "spec-ai-policy"
    "spec-ai-core"
    "spec-ai-api"
    "spec-ai"
    "spec-ai-cli"
)

# Time to wait between publishes for crates.io index to update
WAIT_SECONDS=30

DRY_RUN=false
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "=== DRY RUN MODE ==="
fi

# Get workspace version from root Cargo.toml
VERSION=$(sed -n '/\[workspace\.package\]/,/\[/p' Cargo.toml | grep '^version' | head -1 | sed 's/.*"\(.*\)"/\1/')
echo "Workspace version: $VERSION"
echo ""

# Check if a crate version is already published
is_published() {
    local crate=$1
    local version=$2
    # Query crates.io API to check if version exists
    local status=$(curl -s -o /dev/null -w "%{http_code}" "https://crates.io/api/v1/crates/$crate/$version")
    [[ "$status" == "200" ]]
}

echo "Publishing crates in order:"
for crate in "${CRATES[@]}"; do
    echo "  - $crate"
done
echo ""

published_count=0
skipped_count=0

for i in "${!CRATES[@]}"; do
    crate="${CRATES[$i]}"
    echo "=== $crate ==="

    if is_published "$crate" "$VERSION"; then
        echo "Already published at version $VERSION, skipping..."
        ((skipped_count++))
        echo ""
        continue
    fi

    echo "Publishing $crate@$VERSION..."

    if $DRY_RUN; then
        cargo publish -p "$crate" --dry-run
    else
        cargo publish -p "$crate"
        ((published_count++))

        # Wait between publishes (except for the last one)
        if [[ $i -lt $((${#CRATES[@]} - 1)) ]]; then
            echo "Waiting ${WAIT_SECONDS}s for crates.io index to update..."
            sleep $WAIT_SECONDS
        fi
    fi

    echo ""
done

echo "=== Done ==="
echo "Published: $published_count, Skipped: $skipped_count"
