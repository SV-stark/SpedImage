# End-to-end viewer open benchmark: SpedImage vs the Windows 11 default
# viewer (Photos). Measures time from process launch until a visible window
# whose title contains the image file name appears.
#
# Usage:  pwsh -File scripts/bench_viewers.ps1 [-Runs 7]

param(
    [int]$Runs = 7,
    [string]$CorpusDir = "",
    [string]$SpedImageExe = ""
)

$ErrorActionPreference = "Stop"

if (-not $CorpusDir) {
    $targetDir = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).target_directory
    $CorpusDir = Join-Path $targetDir "bench_corpus"
}
if (-not $SpedImageExe) {
    $targetDir = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).target_directory
    $SpedImageExe = Join-Path $targetDir "release\spedimage.exe"
}

Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class Win32 {
    public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint msg, IntPtr w, IntPtr l);
    static IntPtr found; static string needle;
    static bool Cb(IntPtr h, IntPtr l) {
        if (IsWindowVisible(h)) {
            var sb = new StringBuilder(256); GetWindowText(h, sb, 256);
            if (sb.ToString().ToLower().Contains(needle)) { found = h; return false; }
        }
        return true;
    }
    public static IntPtr Find(string titleNeedle) { needle = titleNeedle; found = IntPtr.Zero; EnumWindows(Cb, IntPtr.Zero); return found; }
    public static void Close(IntPtr h) { PostMessage(h, 0x0010, IntPtr.Zero, IntPtr.Zero); } // WM_CLOSE
}
"@

function Measure-Open {
    param([string]$Image, [ValidateSet("spedimage","default")][string]$Viewer, [int]$TimeoutMs = 30000)

    # Ensure no previous SpedImage instance holds the single-instance lock.
    taskkill /IM spedimage.exe /F 2>$null | Out-Null
    Get-Process PhotosApp*, ApplicationFrameHost -ErrorAction SilentlyContinue |
        Where-Object { $_.MainWindowTitle -ne "" } | ForEach-Object {
            try { $_.CloseMainWindow() | Out-Null } catch {}
        }
    Start-Sleep -Milliseconds 400

    $stem = [IO.Path]::GetFileName($Image).ToLower()
    $sw = [Diagnostics.Stopwatch]::StartNew()
    if ($Viewer -eq "spedimage") {
        Start-Process -FilePath $SpedImageExe -ArgumentList "`"$Image`"" | Out-Null
    } else {
        Start-Process -FilePath $Image | Out-Null   # default handler (Photos)
    }

    while ($sw.ElapsedMilliseconds -lt $TimeoutMs) {
        if ([Win32]::Find($stem) -ne [IntPtr]::Zero) { break }
        Start-Sleep -Milliseconds 5
    }
    $ms = $sw.ElapsedMilliseconds

    $hwnd2 = [Win32]::Find($stem)
    if ($hwnd2 -ne [IntPtr]::Zero) { [Win32]::Close($hwnd2) | Out-Null }
    Start-Sleep -Milliseconds 250
    taskkill /IM spedimage.exe /F 2>$null | Out-Null

    return $ms
}

$cases = @(
    @{ Ext = "JPEG"; File = "bench.jpg" },
    @{ Ext = "PNG";  File = "bench.png" },
    @{ Ext = "TIFF"; File = "bench.tiff" },
    @{ Ext = "GIF";  File = "bench.gif" },
    @{ Ext = "HEIC"; File = "bench.heic" }
)

$rows = foreach ($c in $cases) {
    $img = Join-Path $CorpusDir $c.File
    if (-not (Test-Path $img)) { Write-Warning "missing $img"; continue }
    foreach ($viewer in @("spedimage", "default")) {
        Measure-Open -Image $img -Viewer $viewer | Out-Null   # discard warm-up
        $times = @(1..$Runs | ForEach-Object { Measure-Open -Image $img -Viewer $viewer })
        $sorted = $times | Sort-Object
        [pscustomobject]@{
            Format = $c.Ext
            Viewer = $(if ($viewer -eq "spedimage") { "SpedImage" } else { "Photos (default)" })
            MedianMs = $sorted[[int][math]::Floor($Runs / 2)]
            MinMs = $sorted[0]
            MaxMs = $sorted[-1]
            All = ($times -join ", ")
        }
        Write-Host ("{0,-5} {1,-18} median {2,5} ms  (min {3}, max {4})" -f `
            $c.Ext, $(if ($viewer -eq "spedimage") {"SpedImage"} else {"Photos"}), `
            $sorted[[int][math]::Floor($Runs/2)], $sorted[0], $sorted[-1])
    }
}

$rows | Format-Table Format, Viewer, MedianMs, MinMs, MaxMs -AutoSize
$rows | ConvertTo-Json -Depth 3 | Set-Content "$CorpusDir/viewer_results.json"
Write-Host "Results written to $CorpusDir/viewer_results.json"
