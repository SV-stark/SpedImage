param([string]$Image="H:\target\bench_corpus\bench.jpg")
Add-Type @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class Win32 {
    public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder sb, int max);
    public static string GetTitle(IntPtr h) { var sb=new StringBuilder(256); GetWindowText(h,sb,256); return sb.ToString(); }
}
"@

$exe="H:\target\release\spedimage.exe"
taskkill /IM spedimage.exe /F 2>$null | Out-Null
Start-Sleep -Milliseconds 500
Remove-Item H:\target\startup.log -ErrorAction SilentlyContinue
$sw=[Diagnostics.Stopwatch]::StartNew()
$p=Start-Process -FilePath $exe -ArgumentList "`"$Image`"" -PassThru
$foundAt=$null; $visibleAt=$null
for ($i=0; $i -lt 200; $i++) {
    Start-Sleep -Milliseconds 10
    $ms=$sw.ElapsedMilliseconds
    # Check for any SpedImage window
    $found=$false; $title=""
    [Win32]::EnumWindows({ param($h,$l)
        if ([Win32]::IsWindowVisible($h)) {
            $t=[Win32]::GetTitle($h)
            if ($t -like "*SpedImage*") { $script:foundTitle=$t; return $false }
        }
        return $true
    }, [IntPtr]::Zero) | Out-Null
    # Simpler: just poll EnumWindows via our previous Find logic
    Add-Type -MemberDefinition @"
        [System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsDelegate cb, System.IntPtr l);
        [System.Runtime.InteropServices.DllImport("user32.dll")] public static extern bool IsWindowVisible(System.IntPtr h);
        [System.Runtime.InteropServices.DllImport("user32.dll", CharSet=System.Runtime.InteropServices.CharSet.Unicode)] public static extern int GetWindowText(System.IntPtr h, System.Text.StringBuilder sb, int m);
        public delegate bool EnumWindowsDelegate(System.IntPtr h, System.IntPtr l);
"@ -Name Tmp -Namespace Tmp2 -ErrorAction SilentlyContinue | Out-Null
    # Use our Win32.Find for "bench.jpg"
    $hwnd=[Win32]::Find("bench.jpg")
    if ($hwnd -ne [IntPtr]::Zero -and $null -eq $foundAt) { $foundAt=$ms; "FOUND bench.jpg window at $ms ms hwnd $hwnd title $([Win32]::GetTitle($hwnd))" }
    # Also check any SpedImage window
    $any=[Win32]::Find("spedimage")
    if ($any -ne [IntPtr]::Zero -and $null -eq $visibleAt) { $visibleAt=$ms; "VISIBLE SpedImage at $ms ms title $([Win32]::GetTitle($any))" }
    if ($p.HasExited) { "process exited at $ms"; break }
    if ($ms -gt 3000) { break }
}
"foundAt=$foundAt visibleAt=$visibleAt"
Get-Content H:\target\startup.log -ErrorAction SilentlyContinue | Select-Object -Last 20
taskkill /PID $p.Id /F 2>$null | Out-Null
