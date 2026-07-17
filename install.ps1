$ErrorActionPreference = "Stop"

$Repo = "pr4j3sh/hippo"
$BinaryName = "hippo"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "hippo" }
$Force = $args -contains "--force" -or $args -contains "-f"

function Write-Info($msg)  { Write-Host "==> $msg" -ForegroundColor Green }
function Write-Warn($msg)  { Write-Host "==> $msg" -ForegroundColor Yellow }
function Write-Err($msg)   { Write-Host "==> $msg" -ForegroundColor Red }

function Get-Arch {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64"   { return "x86_64" }
        "ARM64"   { return "aarch64" }
        default   { Write-Err "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE"; exit 1 }
    }
}

function Get-LatestVersion {
    $url = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $release = Invoke-RestMethod -Uri $url -UseBasicParsing
        return $release.tag_name
    } catch {
        Write-Err "Failed to get latest release version"
        exit 1
    }
}

function Get-CurrentVersion {
    $exePath = Join-Path $InstallDir "$BinaryName.exe"
    if (-not (Test-Path $exePath)) { return "" }
    try {
        $output = & $exePath --version 2>$null
        if ($output -match 'hippo\s+(.+)') { return $Matches[1] }
    } catch {}
    return ""
}

function Compare-Versions {
    param([string]$A, [string]$B)
    $aClean = $A -replace '^v', ''
    $bClean = $B -replace '^v', ''
    $aParts = $aClean -split '\.'
    $bParts = $bClean -split '\.'
    for ($i = 0; $i -lt 3; $i++) {
        $aNum = [int]($aParts[$i] ?? 0)
        $bNum = [int]($bParts[$i] ?? 0)
        if ($aNum -gt $bNum) { return 1 }
        if ($aNum -lt $bNum) { return -1 }
    }
    return 0
}

function Install-Binary {
    param([string]$Version, [string]$Arch)

    $assetName = "$BinaryName-windows-$Arch.zip"
    $downloadUrl = "https://github.com/$Repo/releases/download/$Version/$assetName"

    Write-Info "Downloading $BinaryName $Version (windows/$Arch)..."

    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null

    try {
        $zipPath = Join-Path $tmpDir $assetName
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -UseBasicParsing

        Write-Info "Extracting..."
        Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

        Write-Info "Installing to $InstallDir..."
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

        $exePath = Join-Path $tmpDir "$BinaryName.exe"
        if (-not (Test-Path $exePath)) {
            Write-Err "Expected $BinaryName.exe not found in archive"
            exit 1
        }

        Copy-Item $exePath (Join-Path $InstallDir "$BinaryName.exe") -Force
        Write-Info "Installed $BinaryName to $InstallDir\$BinaryName.exe"
    } finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

function Test-AlreadyInstalled {
    $exePath = Join-Path $InstallDir "$BinaryName.exe"
    if (-not (Test-Path $exePath)) { return }

    $current = Get-CurrentVersion

    if ([string]::IsNullOrEmpty($current)) {
        Write-Warn "$BinaryName is already installed (version unknown)"
        if (-not $Force) {
            $confirm = Read-Host "Overwrite? [y/N]"
            if ($confirm -notmatch "^[Yy]$") {
                Write-Info "Aborted."
                exit 0
            }
        }
        return
    }

    $latest = Get-LatestVersion
    $cmp = Compare-Versions -A $current -B $latest

    if ($cmp -eq 0) {
        Write-Info "Already up to date ($current)."
        exit 0
    }

    Write-Info "Updating $BinaryName $current → $latest..."
}

function Test-PathEntry {
    $pathDirs = $env:PATH -split ";"
    if ($pathDirs -notcontains $InstallDir) {
        Write-Warn "$InstallDir is not in your PATH"
        Write-Warn "Add it by running:"
        Write-Warn ""
        Write-Warn "  [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallDir', 'User')"
        Write-Warn ""
    }
}

$arch = Get-Arch
Test-AlreadyInstalled
$version = Get-LatestVersion
Install-Binary -Version $version -Arch $arch
Test-PathEntry
Write-Info "Done! Run '$BinaryName' to start."
