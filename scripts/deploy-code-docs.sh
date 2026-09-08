#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_DIR"

echo "Using project directory: $PROJECT_DIR"

if ! git rev-parse --verify origin/gh-pages >/dev/null 2>&1; then
    echo "Branch 'gh-pages' not found on remote. Creating orphan branch..."
    CURRENT_BRANCH=$(git branch --show-current)
    git checkout --orphan gh-pages
    git rm -rf . >/dev/null
    git commit --allow-empty -m "Initial gh-pages commit"
    git push origin gh-pages
    git checkout "$CURRENT_BRANCH"
fi

echo "Generating documentation..."

cargo clean --doc
cargo docs

echo "Preparing documentation..."
TEMP_DOCS="/tmp/thrust-docs-build"
TEMP_WEBSITE_SOURCE="/tmp/thrust-website-source"
TEMP_WEBSITE_BUILD="/tmp/thrust-website-build"
WEBSITE_REPO_URL="${WEBSITE_REPO_URL:-https://github.com/thrustlang/website}"
WEBSITE_SOURCE_DIR="${WEBSITE_SOURCE_DIR:-}"
PYTHON_BIN="${PYTHON_BIN:-python3}"
rm -rf "$TEMP_DOCS"
rm -rf "$TEMP_WEBSITE_BUILD"
cp -r target/doc "$TEMP_DOCS"

echo '<meta http-equiv="refresh" content="0; url=thrustc/index.html">' > "$TEMP_DOCS/index.html"

echo "Preparing website..."
if [ -n "$WEBSITE_SOURCE_DIR" ]; then
    WEBSITE_SOURCE="$WEBSITE_SOURCE_DIR"
else
    rm -rf "$TEMP_WEBSITE_SOURCE"
    git clone --depth 1 "$WEBSITE_REPO_URL" "$TEMP_WEBSITE_SOURCE"
    WEBSITE_SOURCE="$TEMP_WEBSITE_SOURCE"
fi
"$PYTHON_BIN" "$WEBSITE_SOURCE/scripts/build_subpath.py" --source "$WEBSITE_SOURCE" --base-path /website --output "$TEMP_WEBSITE_BUILD"

echo "Deploying to GitHub Pages..."
PAGES_WORKTREE="/tmp/thrust-gh-pages"
rm -rf "$PAGES_WORKTREE"

git fetch origin gh-pages >/dev/null 2>&1
git worktree add "$PAGES_WORKTREE" gh-pages

pushd "$PAGES_WORKTREE" > /dev/null
    find . -maxdepth 1 ! -name '.git' ! -name '.' ! -name 'website' -exec rm -rf {} +
    rm -rf website
    touch .nojekyll
    
    cp -r "$TEMP_DOCS"/* ./
    cp -r "$TEMP_WEBSITE_BUILD" website
    
    git add -A
    if git diff-index --quiet HEAD --; then
        echo "No changes to documentation."
    else
        git commit -m "Update documentation $(date '+%Y-%m-%d %H:%M')"
        git push origin gh-pages
    fi
popd > /dev/null

git worktree remove "$PAGES_WORKTREE"
rm -rf "$TEMP_DOCS"
rm -rf "$TEMP_WEBSITE_SOURCE"
rm -rf "$TEMP_WEBSITE_BUILD"

echo "Done. Documentation updated successfully."
