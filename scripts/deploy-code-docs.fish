#!/usr/bin/fish

set -e

set SCRIPT_DIR (realpath (dirname (status filename)))
set PROJECT_DIR (realpath "$SCRIPT_DIR/..")

cd $PROJECT_DIR
echo "Using project directory: $PROJECT_DIR"

if not git rev-parse --verify origin/gh-pages >/dev/null 2>&1
    echo "Branch 'gh-pages' not found on remote. Creating..."
    set CURRENT_BRANCH (git branch --show-current)
    git checkout --orphan gh-pages
    git rm -rf .
    git commit --allow-empty -m "Initial gh-pages commit"
    git push origin gh-pages
    git checkout $CURRENT_BRANCH
end

echo "Generating documentation..."
cargo clean --doc
cargo docs

echo "Preparing documentation..."
set TEMP_DOCS "/tmp/thrust-docs-build"
set TEMP_WEBSITE_SOURCE "/tmp/thrust-website-source"
set TEMP_WEBSITE_BUILD "/tmp/thrust-website-build"
if not set -q WEBSITE_REPO_URL
    set WEBSITE_REPO_URL "https://github.com/thrustlang/website"
end

if not set -q PYTHON_BIN
    set PYTHON_BIN "python3"
end

rm -rf $TEMP_DOCS
rm -rf $TEMP_WEBSITE_SOURCE
rm -rf $TEMP_WEBSITE_BUILD
cp -r target/doc $TEMP_DOCS

echo '<meta http-equiv="refresh" content="0; url=thrustc/index.html">' > "$TEMP_DOCS/index.html"

echo "Preparing website..."
git clone --depth 1 $WEBSITE_REPO_URL $TEMP_WEBSITE_SOURCE
$PYTHON_BIN "$TEMP_WEBSITE_SOURCE/scripts/build_subpath.py" --source $TEMP_WEBSITE_SOURCE --base-path /website --output $TEMP_WEBSITE_BUILD

echo "Deploying to GitHub Pages..."
set PAGES_WORKTREE "/tmp/thrust-gh-pages"
rm -rf $PAGES_WORKTREE

git fetch origin gh-pages
git worktree add $PAGES_WORKTREE gh-pages

pushd $PAGES_WORKTREE
    find . -maxdepth 1 ! -name '.git' ! -name '.' ! -name 'website' -exec rm -rf {} +
    rm -rf website
    cp -r $TEMP_DOCS/* ./
    cp -r $TEMP_WEBSITE_BUILD website
    
    git add -A
    if git diff-index --quiet HEAD --
        echo "No changes to documentation."
    else
        git commit -m "Update documentation (date '+%Y-%m-%d %H:%M')"
        git push origin gh-pages
    end
popd

git worktree remove $PAGES_WORKTREE
rm -rf $TEMP_DOCS
rm -rf $TEMP_WEBSITE_SOURCE
rm -rf $TEMP_WEBSITE_BUILD
echo "Done."
