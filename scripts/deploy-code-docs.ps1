$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = (Get-Item (Join-Path $ScriptDir "..")).FullName

Set-Location $ProjectDir
Write-Host "Using project directory: $ProjectDir" -ForegroundColor Cyan

$remoteBranch = git ls-remote --heads origin gh-pages
if (-not $remoteBranch) {
    Write-Host "Branch 'gh-pages' not found. Creating..." -ForegroundColor Yellow
    $currentBranch = git branch --show-current
    git checkout --orphan gh-pages
    git rm -rf .
    git commit --allow-empty -m "Initial gh-pages commit"
    git push origin gh-pages
    git checkout $currentBranch
}

Write-Host "Generating documentation..."
cargo clean --doc
cargo docs

Write-Host "Preparing documentation..."
$TempDocs = Join-Path $env:TEMP "thrust-docs-build"
$TempWebsiteSource = Join-Path $env:TEMP "thrust-website-source"
$TempWebsiteBuild = Join-Path $env:TEMP "thrust-website-build"
$WebsiteRepoUrl = if ($env:WEBSITE_REPO_URL) { $env:WEBSITE_REPO_URL } else { "https://github.com/thrustlang/website" }
$PythonBin = if ($env:PYTHON_BIN) { $env:PYTHON_BIN } else { "python" }
if (Test-Path $TempDocs) { Remove-Item -Recurse -Force $TempDocs }
if (Test-Path $TempWebsiteSource) { Remove-Item -Recurse -Force $TempWebsiteSource }
if (Test-Path $TempWebsiteBuild) { Remove-Item -Recurse -Force $TempWebsiteBuild }
Copy-Item -Path "target/doc" -Destination $TempDocs -Recurse

Set-Content -Path (Join-Path $TempDocs "index.html") -Value '<meta http-equiv="refresh" content="0; url=thrustc/index.html">'

Write-Host "Preparing website..."
git clone --depth 1 $WebsiteRepoUrl $TempWebsiteSource
& $PythonBin (Join-Path $TempWebsiteSource "scripts/build_subpath.py") --source $TempWebsiteSource --base-path /website --output $TempWebsiteBuild

Write-Host "Deploying to GitHub Pages..."
$PagesWorktree = Join-Path $env:TEMP "thrust-gh-pages"
if (Test-Path $PagesWorktree) { Remove-Item -Recurse -Force $PagesWorktree }

git fetch origin gh-pages
git worktree add $PagesWorktree gh-pages

Push-Location $PagesWorktree
    Get-ChildItem -Exclude .git | Remove-Item -Recurse -Force
    
    Copy-Item -Path "$TempDocs\*" -Destination "." -Recurse
    New-Item -ItemType Directory -Path "website" -Force | Out-Null
    Copy-Item -Path "$TempWebsiteBuild\*" -Destination "website" -Recurse
    
    git add -A
    if (git diff-index --quiet HEAD --) {
        Write-Host "No changes to documentation."
    } else {
        $date = Get-Date -Format "yyyy-MM-dd HH:mm"
        git commit -m "Update documentation $date"
        git push origin gh-pages
    }
Pop-Location

git worktree remove $PagesWorktree
Remove-Item -Recurse -Force $TempDocs
Remove-Item -Recurse -Force $TempWebsiteSource
Remove-Item -Recurse -Force $TempWebsiteBuild
Write-Host "Done." -ForegroundColor Green
