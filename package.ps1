<#
.SYNOPSIS
    Anadil V0.1 release packaging script.

.DESCRIPTION
    Bu script Anadil binary'lerini derler, runtime kutuphanesini
    onceden assemble eder, distribution klasor yapisini olusturur ve
    Anadil-vX.Y.Z-windows-x64.zip arsivini uretir.

    Tek dogruluk kaynagi: Docs/release_layout.md

.EXAMPLE
    PS> .\package.ps1
    Varsayilan calistirma. cargo build, ml64, lib, Compress-Archive
    sirasiyla calistirilir; ZIP dosyasi target\dist altina yazilir.

.EXAMPLE
    PS> .\package.ps1 -SkipBuild
    Cargo build adimini atlar (mevcut target\release ciktilarini
    kullanir). Hizli iterasyon icin.

.NOTES
    Visual Studio Build Tools (ml64, lib) ile calisan ortamda
    calistirilmali. PATH'te yoksa vcvars64.bat'in cagirildigi bir
    Developer Command Prompt'tan PowerShell baslatin veya $env:PATH'e
    gerekli yolları ekleyin.
#>

[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$Installer
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

# --- Surum bilgisini Cargo.toml'dan oku ----------------------------------

function Get-AnadilVersion {
    $cargoToml = Join-Path $RepoRoot "Cargo.toml"
    $line = Select-String -Path $cargoToml -Pattern '^version\s*=' | Select-Object -First 1
    if (-not $line) {
        throw "Cargo.toml icinde version satiri bulunamadi."
    }
    if ($line.Line -notmatch '"([^"]+)"') {
        throw "Cargo.toml version satiri ayristirilamadi: $($line.Line)"
    }
    return $Matches[1]
}

# --- Build Tools tespiti -------------------------------------------------

function Get-VcVars64 {
    if ((Get-Command ml64.exe -ErrorAction SilentlyContinue) -and
        (Get-Command lib.exe  -ErrorAction SilentlyContinue)) {
        return $null  # PATH'te zaten var; vcvars64'e gerek yok
    }

    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $vsRoot = & $vswhere -latest -prerelease -products "*" `
            -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
            -property installationPath 2>$null
        if ($vsRoot) {
            $vcvars = Join-Path $vsRoot "VC\Auxiliary\Build\vcvars64.bat"
            if (Test-Path $vcvars) { return $vcvars }
        }
    }

    $candidates = @(
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        "${env:ProgramFiles}\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
        "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
    )
    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) { return $candidate }
    }

    throw "Visual Studio Build Tools bulunamadi. ml64.exe ve lib.exe gerekli."
}

function Invoke-MsvcTool {
    param(
        [Parameter(Mandatory)] [string]$Tool,
        [Parameter(Mandatory)] [string[]]$ToolArgs,
        [string]$VcVars64
    )

    if (-not $VcVars64) {
        & $Tool @ToolArgs
        if ($LASTEXITCODE -ne 0) {
            throw "$Tool basarisiz oldu (exit $LASTEXITCODE)."
        }
        return
    }

    $argString = ($ToolArgs | ForEach-Object { "`"$_`"" }) -join ' '
    $batLines = @(
        '@echo off',
        "call `"$VcVars64`" >nul",
        "$Tool $argString",
        "exit /b %ERRORLEVEL%"
    )
    $batPath = Join-Path $env:TEMP ("anadil-package-" + [Guid]::NewGuid().ToString("N") + ".bat")
    Set-Content -Path $batPath -Value $batLines -Encoding ASCII

    try {
        & cmd.exe /d /c $batPath
        if ($LASTEXITCODE -ne 0) {
            throw "$Tool basarisiz oldu (vcvars64 uzerinden, exit $LASTEXITCODE)."
        }
    } finally {
        Remove-Item -Force -ErrorAction SilentlyContinue $batPath
    }
}

# --- Ana akis ------------------------------------------------------------

Push-Location $RepoRoot
try {
    $version = Get-AnadilVersion
    $distName = "Anadil-v$version-windows-x64"
    Write-Host "==> Anadil v$version paketleniyor..." -ForegroundColor Cyan

    # 1. Cargo build (release)
    if (-not $SkipBuild) {
        Write-Host "==> cargo build --release --bin anadil"
        cargo build --release --bin anadil
        if ($LASTEXITCODE -ne 0) { throw "cargo build (anadil) basarisiz." }

        Write-Host "==> cargo build --release --bin anadil-ide"
        cargo build --release --bin anadil-ide
        if ($LASTEXITCODE -ne 0) { throw "cargo build (anadil-ide) basarisiz." }
    } else {
        Write-Host "==> -SkipBuild: cargo adimi atlandi" -ForegroundColor Yellow
    }

    $anadilExe    = Join-Path $RepoRoot "target\release\anadil.exe"
    $anadilIdeExe = Join-Path $RepoRoot "target\release\anadil-ide.exe"
    foreach ($exe in @($anadilExe, $anadilIdeExe)) {
        if (-not (Test-Path $exe)) { throw "Beklenen binary bulunamadi: $exe" }
    }

    # 2. Pre-built runtime.lib
    Write-Host "==> Pre-built runtime kutuphanesi olusturuluyor..."
    $vcVars64 = Get-VcVars64
    if ($vcVars64) {
        Write-Host "    vcvars64: $vcVars64"
    } else {
        Write-Host "    Build Tools PATH'te zaten mevcut"
    }

    $runtimeAsm = Join-Path $RepoRoot "runtime\anadil_runtime.asm"
    if (-not (Test-Path $runtimeAsm)) { throw "Runtime asm bulunamadi: $runtimeAsm" }

    $runtimeBuildDir = Join-Path $RepoRoot "target\release-runtime"
    New-Item -ItemType Directory -Force -Path $runtimeBuildDir | Out-Null
    $runtimeObj = Join-Path $runtimeBuildDir "anadil_runtime.obj"
    $runtimeLib = Join-Path $runtimeBuildDir "anadil_runtime.lib"

    Invoke-MsvcTool -Tool "ml64" -VcVars64 $vcVars64 -ToolArgs @(
        "/nologo", "/c", "/Fo$runtimeObj", $runtimeAsm
    )
    Invoke-MsvcTool -Tool "lib" -VcVars64 $vcVars64 -ToolArgs @(
        "/nologo", "/OUT:$runtimeLib", $runtimeObj
    )
    if (-not (Test-Path $runtimeLib)) { throw "Runtime lib uretilemedi: $runtimeLib" }

    # 3. Dist klasoru
    $distRoot = Join-Path $RepoRoot "target\dist"
    $distDir  = Join-Path $distRoot $distName
    if (Test-Path $distRoot) { Remove-Item -Recurse -Force $distRoot }
    New-Item -ItemType Directory -Force -Path $distDir | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $distDir "runtime")  | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $distDir "examples") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $distDir "docs")     | Out-Null

    # 4. Dosya kopyalama
    Write-Host "==> Distribution dosyalari kopyalaniyor..."
    Copy-Item $anadilExe    (Join-Path $distDir "anadil.exe")
    Copy-Item $anadilIdeExe (Join-Path $distDir "anadil-ide.exe")
    Copy-Item $runtimeAsm   (Join-Path $distDir "runtime\anadil_runtime.asm")
    Copy-Item $runtimeLib   (Join-Path $distDir "runtime\anadil_runtime.lib")

    Get-ChildItem (Join-Path $RepoRoot "examples") -Filter "*.ana" |
        Copy-Item -Destination (Join-Path $distDir "examples")

    $docMappings = @{
        "Docs\proje_raporu.md"   = "docs\PROJE_RAPORU.md"
        "Docs\dil_referansi.md"  = "docs\DIL_REFERANSI.md"
        "Docs\native_compiler.md" = "docs\NATIVE_COMPILER.md"
    }
    foreach ($kvp in $docMappings.GetEnumerator()) {
        $src = Join-Path $RepoRoot $kvp.Key
        $dst = Join-Path $distDir $kvp.Value
        if (Test-Path $src) {
            Copy-Item $src $dst
        } else {
            Write-Warning "Beklenen belge bulunamadi: $src (atlandi)"
        }
    }

    foreach ($name in @("KURULUM.txt", "CHANGELOG.txt")) {
        $src = Join-Path $RepoRoot $name
        if (Test-Path $src) {
            Copy-Item $src (Join-Path $distDir $name)
        } else {
            Write-Warning "$name bulunamadi (atlandi)"
        }
    }

    $licenseSrc = Join-Path $RepoRoot "LICENSE"
    if (Test-Path $licenseSrc) {
        Copy-Item $licenseSrc (Join-Path $distDir "LICENSE.txt")
    } else {
        Write-Warning "LICENSE dosyasi bulunamadi; LICENSE.txt eklenmedi."
    }

    # 5. Dinamik README.txt
    $readmeContent = @"
Anadil v$version
Windows x64

Bu paket Anadil komut satiri arayuzunu (anadil.exe), native IDE'yi
(anadil-ide.exe), runtime kutuphanesini ve ornekleri icerir.

Hizli Baslangic
---------------

  1. Bu klasoru tercih ettiginiz bir konuma cikartin.
  2. anadil-ide.exe ile IDE'yi acin, ya da
     komut isteminde: anadil.exe yardim

Ayrintili kurulum:  KURULUM.txt
Surum notlari:      CHANGELOG.txt
Belgeler:           docs/

Kaynak:  https://github.com/ArsenAlighieri/Anadil
"@
    Set-Content -Path (Join-Path $distDir "README.txt") -Value $readmeContent -Encoding UTF8

    # 6. Zip + SHA256
    Write-Host "==> ZIP olusturuluyor..."
    $zipPath = Join-Path $distRoot "$distName.zip"
    Compress-Archive -Path (Join-Path $distDir "*") -DestinationPath $zipPath -Force

    $hash = Get-FileHash -Algorithm SHA256 $zipPath
    $sizeBytes = (Get-Item $zipPath).Length
    $sizeMb = [math]::Round($sizeBytes / 1MB, 2)

    Write-Host ""
    Write-Host "==> ZIP hazir." -ForegroundColor Green
    Write-Host "    Klasor:  $distDir"
    Write-Host "    Arsiv:   $zipPath"
    Write-Host "    Boyut:   $sizeMb MB"
    Write-Host "    SHA256:  $($hash.Hash)"

    # 7. Opsiyonel installer (NSIS)
    if ($Installer) {
        Write-Host ""
        Write-Host "==> NSIS installer olusturuluyor..." -ForegroundColor Cyan

        $makeNsis = Get-Command makensis.exe -ErrorAction SilentlyContinue
        if (-not $makeNsis) {
            $candidates = @(
                "${env:ProgramFiles(x86)}\NSIS\makensis.exe",
                "${env:ProgramFiles}\NSIS\makensis.exe"
            )
            foreach ($candidate in $candidates) {
                if (Test-Path $candidate) {
                    $makeNsis = Get-Item $candidate
                    break
                }
            }
        }
        if (-not $makeNsis) {
            throw "NSIS bulunamadi. Indirme: https://nsis.sourceforge.io/Download"
        }
        $makeNsisDisplay = if ($makeNsis.Source) { $makeNsis.Source } else { $makeNsis.FullName }
        Write-Host "    makensis: $makeNsisDisplay"

        $nsiScript = Join-Path $RepoRoot "installer.nsi"
        if (-not (Test-Path $nsiScript)) {
            throw "installer.nsi bulunamadi: $nsiScript"
        }

        # NSIS'e versiyonu ve dist klasorunu defines ile gec
        $distRelative   = "target\dist\$distName"
        $setupRelative  = "target\dist\Anadil-Setup-v$version.exe"
        $defines = @(
            "/DANADIL_VERSION=$version",
            "/DANADIL_DIST_DIR=$distRelative",
            "/DANADIL_OUTFILE=$setupRelative"
        )

        $makeNsisPath = if ($makeNsis.Source) { $makeNsis.Source } else { $makeNsis.FullName }
        & $makeNsisPath @defines $nsiScript
        if ($LASTEXITCODE -ne 0) {
            throw "makensis basarisiz oldu (exit $LASTEXITCODE)."
        }

        $setupPath = Join-Path $RepoRoot $setupRelative
        if (-not (Test-Path $setupPath)) {
            throw "Setup uretimi tamamlanmadi: $setupPath"
        }
        $setupHash = Get-FileHash -Algorithm SHA256 $setupPath
        $setupSizeMb = [math]::Round((Get-Item $setupPath).Length / 1MB, 2)

        Write-Host ""
        Write-Host "==> Setup hazir." -ForegroundColor Green
        Write-Host "    Setup:   $setupPath"
        Write-Host "    Boyut:   $setupSizeMb MB"
        Write-Host "    SHA256:  $($setupHash.Hash)"
    }

    Write-Host ""
    Write-Host "GitHub Releases'a yuklerken release notes icine SHA256(lari) ekleyin."
}
finally {
    Pop-Location
}
