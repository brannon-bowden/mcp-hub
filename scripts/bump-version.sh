#!/bin/bash
#
# bump-version.sh - Update version across all project files
#
# Usage:
#   ./scripts/bump-version.sh patch      # 0.1.5 -> 0.1.6
#   ./scripts/bump-version.sh minor      # 0.1.5 -> 0.2.0
#   ./scripts/bump-version.sh major      # 0.1.5 -> 1.0.0
#   ./scripts/bump-version.sh 0.2.0      # Set specific version
#
# Options:
#   --commit    Commit the version changes
#   --tag       Create a git tag (implies --commit)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Find project root (where package.json is)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse arguments
BUMP_TYPE=""
DO_COMMIT=false
DO_TAG=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --commit)
            DO_COMMIT=true
            shift
            ;;
        --tag)
            DO_TAG=true
            DO_COMMIT=true  # tag implies commit
            shift
            ;;
        patch|minor|major|[0-9]*)
            BUMP_TYPE="$1"
            shift
            ;;
        *)
            echo -e "${RED}Unknown argument: $1${NC}"
            echo "Usage: $0 <patch|minor|major|x.y.z> [--commit] [--tag]"
            exit 1
            ;;
    esac
done

if [ -z "$BUMP_TYPE" ]; then
    echo -e "${RED}Error: Version bump type required${NC}"
    echo "Usage: $0 <patch|minor|major|x.y.z> [--commit] [--tag]"
    exit 1
fi

# Get current version from package.json
CURRENT_VERSION=$(grep '"version"' "$PROJECT_ROOT/package.json" | head -1 | sed 's/.*"\([0-9]*\.[0-9]*\.[0-9]*\)".*/\1/')
echo -e "Current version: ${YELLOW}$CURRENT_VERSION${NC}"

# Calculate new version
if [[ "$BUMP_TYPE" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    # Specific version provided
    NEW_VERSION="$BUMP_TYPE"
else
    # Bump type provided
    IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

    case "$BUMP_TYPE" in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch)
            PATCH=$((PATCH + 1))
            ;;
    esac

    NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"
fi

echo -e "New version:     ${GREEN}$NEW_VERSION${NC}"

# Update package.json
sed -i.bak "s/\"version\": \"[0-9]*\.[0-9]*\.[0-9]*\"/\"version\": \"$NEW_VERSION\"/" "$PROJECT_ROOT/package.json"
rm -f "$PROJECT_ROOT/package.json.bak"
echo -e "${GREEN}✓${NC} Updated package.json"

# Update Cargo.toml
sed -i.bak "s/^version = \"[0-9]*\.[0-9]*\.[0-9]*\"/version = \"$NEW_VERSION\"/" "$PROJECT_ROOT/src-tauri/Cargo.toml"
rm -f "$PROJECT_ROOT/src-tauri/Cargo.toml.bak"
echo -e "${GREEN}✓${NC} Updated src-tauri/Cargo.toml"

# Update tauri.conf.json
sed -i.bak "s/\"version\": \"[0-9]*\.[0-9]*\.[0-9]*\"/\"version\": \"$NEW_VERSION\"/" "$PROJECT_ROOT/src-tauri/tauri.conf.json"
rm -f "$PROJECT_ROOT/src-tauri/tauri.conf.json.bak"
echo -e "${GREEN}✓${NC} Updated src-tauri/tauri.conf.json"

# Update Cargo.lock by running cargo check
echo "Updating Cargo.lock..."
(cd "$PROJECT_ROOT/src-tauri" && cargo check --quiet 2>/dev/null) || true
echo -e "${GREEN}✓${NC} Updated Cargo.lock"

if [ "$DO_COMMIT" = true ]; then
    echo ""
    echo "Committing changes..."
    git -C "$PROJECT_ROOT" add package.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
    git -C "$PROJECT_ROOT" commit -m "chore: bump version to $NEW_VERSION"
    echo -e "${GREEN}✓${NC} Changes committed"
fi

if [ "$DO_TAG" = true ]; then
    echo "Creating tag v$NEW_VERSION..."
    git -C "$PROJECT_ROOT" tag "v$NEW_VERSION"
    echo -e "${GREEN}✓${NC} Tag v$NEW_VERSION created"
    echo ""
    echo -e "${YELLOW}To push changes and tag:${NC}"
    echo "  git push origin main && git push origin v$NEW_VERSION"
fi

echo ""
echo -e "${GREEN}Version updated to $NEW_VERSION${NC}"

if [ "$DO_COMMIT" = false ]; then
    echo ""
    echo "Files changed (not committed):"
    echo "  - package.json"
    echo "  - src-tauri/Cargo.toml"
    echo "  - src-tauri/Cargo.lock"
    echo "  - src-tauri/tauri.conf.json"
fi
