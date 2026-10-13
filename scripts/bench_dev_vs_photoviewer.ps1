# End-to-end launch latency benchmark: SpedImage (Dev build) vs Windows Photo Viewer
# Tested on 4K UHD JPG and 4K UHD PNG.
# Measures time from process invocation until window is created, rendered, and visible.

param(
    [int]$Runs = 5,
    [string]$DevExe = "H:\target\debug\spedimage.exe",
    [string]$CorpusDir = "E:\SpedImage\target\bench_corpus"
)

$ErrorActionPreference = "Stop"

Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;

public static class BenchWin32 {
    public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder sb, int max);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);

    static IntPtr found;
    static string needle;

    static bool Cb(IntPtr h, IntPtr l) {
        if (IsWindowVisible(h)) {
            var sb = new StringBuilder(256);
            GetWindowText(h, sb, 256);
            if (sb.ToString().ToLower().Contains(needle)) {
                found = h;
                return false;
            }
        }
        return true;
    }

    public static IntPtr FindByTitle(string titleNeedle) {
        needle = titleNeedle.ToLower();
        found = IntPtr.Zero;
        EnumWindows(Cb, IntPtr.Zero);
        return found;
    }

    public static void Close(IntPtr h) {
        PostMessage(h, 0x0010, IntPtr.Zero, IntPtr.Zero); // WM_CLOSE
    }
}
"@

function Cleanup-Processes {
    taskkill /IM spedimage.exe /F 2>$null | Out-Null
    taskkill /IM rundll32.exe /F 2>$null | Out-Null
    Start-Sleep -Milliseconds 300
}

function Measure-ViewerLaunch {
    param(
        [string]$ViewerType, # 'spedimage_dev' or 'photo_viewer'
        [string]$ImagePath,
        [int]$TimeoutMs = 15000
    )

    Cleanup-Processes

    $stem = [System.IO.Path]::GetFileName($ImagePath).ToLower()
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $proc = $null

    if ($ViewerType -eq "spedimage_dev") {
        $proc = Start-Process -FilePath $DevExe -ArgumentList "`"$ImagePath`"" -PassThru
    } elseif ($ViewerType -eq "photo_viewer") {
        $dllPath = "$env:ProgramFiles\Windows Photo Viewer\PhotoViewer.dll"
        $proc = Start-Process -FilePath "rundll32.exe" -ArgumentList "`"$dllPath`",ImageView_Fullscreen $ImagePath" -PassThru
    }

    $hwnd = [IntPtr]::Zero
    while ($sw.ElapsedMilliseconds -lt $TimeoutMs) {
        $hwnd = [BenchWin32]::FindByTitle($stem)
        if ($hwnd -ne [IntPtr]::Zero) { break }
        Start-Sleep -Milliseconds 2
    }
    $sw.Stop()
    $elapsedMs = $sw.ElapsedMilliseconds

    $wsMB = 0
    try {
        if ($proc -and -not $proc.HasExited) {
            $proc.Refresh()
            $wsMB = [Math]::Round($proc.WorkingSet64 / 1MB, 2)
        }
    } catch {}

    if ($hwnd -ne [IntPtr]::Zero) {
        [BenchWin32]::Close($hwnd)
    }
    Start-Sleep -Milliseconds 200
    Cleanup-Processes

    return [PSCustomObject]@{
        ElapsedMs = $elapsedMs
        WorkingSetMB = $wsMB
    }
}

$files = @(
    @{ Format = "JPEG"; Filename = "bench.jpg" },
    @{ Format = "PNG";  Filename = "bench.png" }
)

$viewers = @(
    @{ Key = "spedimage_dev"; Name = "SpedImage (Dev Build)" },
    @{ Key = "photo_viewer";  Name = "Windows Photo Viewer" }
)

Write-Host "==========================================================================" -ForegroundColor Cyan
Write-Host " BENCHMARK: SpedImage (Dev Build) vs Windows Photo Viewer" -ForegroundColor Cyan
Write-Host " Runs: $Runs (+1 warm-up discarded per test) | Target: 4K UHD Images" -ForegroundColor Cyan
Write-Host "==========================================================================" -ForegroundColor Cyan

$results = @()

foreach ($f in $files) {
    $fullPath = Join-Path $CorpusDir $f.Filename
    if (-not (Test-Path $fullPath)) {
        Write-Warning "File not found: $fullPath"
        continue
    }
    $fileInfo = Get-Item $fullPath
    $fileSizeMB = [Math]::Round($fileInfo.Length / 1MB, 2)

    Write-Host "`n--- Testing $($f.Format) ($($f.Filename) - ${fileSizeMB} MB) ---" -ForegroundColor Yellow

    foreach ($v in $viewers) {
        Write-Host "  Warming up $($v.Name)..." -NoNewline
        $null = Measure-ViewerLaunch -ViewerType $v.Key -ImagePath $fullPath
        Write-Host " Done."

        $samples = @()
        $memSamples = @()

        for ($i = 1; $i -le $Runs; $i++) {
            Write-Host "    Run $i/$Runs..." -NoNewline
            $res = Measure-ViewerLaunch -ViewerType $v.Key -ImagePath $fullPath
            $samples += $res.ElapsedMs
            if ($res.WorkingSetMB -gt 0) { $memSamples += $res.WorkingSetMB }
            Write-Host " $($res.ElapsedMs) ms (RAM: $($res.WorkingSetMB) MB)"
        }

        $sorted = $samples | Sort-Object
        $median = $sorted[[int][math]::Floor($Runs / 2)]
        $min = $sorted[0]
        $max = $sorted[-1]
        $mean = [Math]::Round(($samples | Measure-Object -Average).Average, 1)
        $avgMem = if ($memSamples.Count -gt 0) { [Math]::Round(($memSamples | Measure-Object -Average).Average, 1) } else { 0 }

        $results += [PSCustomObject]@{
            Format       = $f.Format
            Image        = $f.Filename
            SizeMB       = $fileSizeMB
            Viewer       = $v.Name
            MinMs        = $min
            MedianMs     = $median
            MeanMs       = $mean
            MaxMs        = $max
            WorkingSetMB = $avgMem
            AllRuns      = ($samples -join ", ")
        }
    }
}

Write-Host "`n==========================================================================" -ForegroundColor Green
Write-Host " FINAL RESULTS SUMMARY" -ForegroundColor Green
Write-Host "==========================================================================" -ForegroundColor Green

$results | Format-Table Format, Image, Viewer, MinMs, MedianMs, MeanMs, MaxMs, WorkingSetMB -AutoSize

$jsonOut = Join-Path $CorpusDir "benchmark_dev_vs_photoviewer.json"
$results | ConvertTo-Json -Depth 3 | Set-Content $jsonOut
Write-Host "`nDetailed results saved to: $jsonOut" -ForegroundColor Cyan
