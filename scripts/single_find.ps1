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
    static IntPtr found; static string needle;
    static bool Cb(IntPtr h, IntPtr l) {
        if (IsWindowVisible(h)) {
            var t=GetTitle(h);
            if (t.ToLower().Contains(needle)) { found=h; return false; }
        }
        return true;
    }
    public static IntPtr Find(string titleNeedle) { needle=titleNeedle.ToLower(); found=IntPtr.Zero; EnumWindows(Cb, IntPtr.Zero); return found; }
    public static System.Collections.Generic.List<string> ListVisible() {
        var list=new System.Collections.Generic.List<string>();
        EnumWindows((h,l)=>{
            if(IsWindowVisible(h)){
                var t=GetTitle(h);
                if(!string.IsNullOrEmpty(t)) list.Add(t);
            }
            return true;
        }, IntPtr.Zero);
        return list;
    }
}
"@

$exe="H:\target\release\spedimage.exe"
taskkill /IM spedimage.exe /F 2>$null | Out-Null
Start-Sleep -Milliseconds 800
Remove-Item H:\target\startup.log -ErrorAction SilentlyContinue
$sw=[Diagnostics.Stopwatch]::StartNew()
$p=Start-Process -FilePath $exe -ArgumentList "`"$Image`"" -PassThru
$foundAt=$null
for ($i=0; $i -lt 600; $i++) {
    Start-Sleep -Milliseconds 5
    $hwnd=[Win32]::Find("bench.jpg")
    if ($hwnd -ne [IntPtr]::Zero -and $null -eq $foundAt) {
        $foundAt=$sw.ElapsedMilliseconds
        "FOUND at $foundAt ms hwnd $hwnd title $([Win32]::GetTitle($hwnd))"
        break
    }
    if ($p.HasExited) { "process exited at $($sw.ElapsedMilliseconds)ms"; break }
}
if ($null -eq $foundAt) {
    "NOT FOUND after $($sw.ElapsedMilliseconds)ms"
    "Visible windows:"
    [Win32]::ListVisible() | Select-Object -First 20 | ForEach-Object { "  $_" }
    Get-Content H:\target\startup.log -ErrorAction SilentlyContinue | Select-Object -Last 20
}
Start-Sleep -Milliseconds 500
Get-Content H:\target\startup.log -ErrorAction SilentlyContinue | Select-Object -Last 20
taskkill /PID $p.Id /F 2>$null | Out-Null
"done foundAt=$foundAt"
