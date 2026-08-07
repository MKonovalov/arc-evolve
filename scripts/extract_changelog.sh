#!/usr/bin/env bash
# Extract changelog section for a specific version tag from CHANGELOG.md
# Usage: ./scripts/extract_changelog.sh v0.1.5
set -euo pipefail

TAG="${1:?Usage: extract_changelog.sh <tag>}"
VERSION="${TAG#v}"

CHANGELOG="$(dirname "$0")/../CHANGELOG.md"

if [ ! -f "$CHANGELOG" ]; then
  echo "Error: CHANGELOG.md not found" >&2
  exit 1
fi

# Extract everything between ## [VERSION] and the next ## [ heading
BODY=$(awk -v ver="$VERSION" '
  /^## \[/ {
    if (found) exit
    if (index($0, "[" ver "]")) { found=1; next }
  }
  found { print }
' "$CHANGELOG")

if [ -z "$BODY" ]; then
  # Fallback: derive a body from git history since the version tag. This keeps
  # releases working even if CHANGELOG.md wasn't updated for this version.
  echo "Warning: Version $VERSION not found in CHANGELOG.md; using git log fallback" >&2
  PREV_TAG="$(git describe --tags --abbrev=0 --match 'v*' "$TAG^" 2>/dev/null || true)"
  if [ -n "$PREV_TAG" ]; then
    BODY="Release $VERSION

Changes since $PREV_TAG:

$(git log --no-merges --pretty=format='- %s' "${PREV_TAG}..${TAG}" 2>/dev/null | head -50)"
  else
    BODY="Release $VERSION

$(git log --no-merges --pretty=format='- %s' "$TAG" 2>/dev/null | head -50)"
  fi
fi

echo "$BODY"
