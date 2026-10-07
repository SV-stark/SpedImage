#!/usr/bin/env pwsh
# Assert the release binary stays under a documented budget.
#
# README used to claim a ~10MB base app size. That claim drifted silently: both
# workflows ran `cargo build --release` and nothing ever looked at the output,
# and it had been wrong for a while. This turns the size into a gate with a
# number that was actually measured, and points README at it.
#
# Usage: pwsh -File scripts/check_size.ps1 [-Path <exe>] [-BudgetMB <n>]

param(
    [string] $Path = "target/release/spedimage.exe",
    # 22MB, measured rather than guessed.
    #
    # Release profile (opt-level 3, lto, codegen-units 1, strip), x86_64-pc-windows-msvc:
    #   23.30 MB at HEAD
    #   18.30 MB after the dependency trim below
    # The 5MB came out of: the AGPL git-pinned heic subtree (rav1d-safe,
    # ultrahdr-core, archmage, magetypes, whereat, safe_unaligned_simd), the
    # duplicate jxl-oxide 0.12 that zune-image's default features dragged in,
    # and the arboard `image` crate with its PNG + TIFF decoders.
    #
    # So README's old ~10MB claim was wrong by more than half even before this
    # work, and nothing checked it. 22MB leaves ~3.7MB of headroom over the
    # current binary and would have failed on HEAD, which is the point. wgpu
    # plus its naga shaders are most of what remains, so this ceiling moves
    # when wgpu does. Raise it deliberately, with a note on what grew.
    [double] $BudgetMB = 22.0
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $Path)) {
    Write-Host "FAIL: $Path not found. Build --release before running this."
    exit 1
}

$bytes = (Get-Item $Path).Length
$mb = $bytes / 1MB

"{0,-28} {1,10:N2} MB   (budget {2:N2} MB)" -f "spedimage.exe", $mb, $BudgetMB | Write-Host

if ($mb -gt $BudgetMB) {
    # Write-Host, not Write-Error: with $ErrorActionPreference = "Stop" a
    # Write-Error throws, which dumps a red PowerShell exception record over
    # the message and makes the real cause hard to read in CI logs.
    Write-Host "FAIL: binary is $([math]::Round($mb, 2)) MB, over the $([math]::Round($BudgetMB, 2)) MB budget."
    Write-Host "A dependency was probably added without thinking. Run 'cargo tree -d' to see what is compiled twice."
    Write-Host "If the growth is intentional, raise -BudgetMB here and update the size claim in README.md."
    exit 1
}

Write-Host "OK: within budget."
