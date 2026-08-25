$ErrorActionPreference = "Stop"
$failed = @()
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

git ls-files | ForEach-Object {
    $path = $_
    $ext = [IO.Path]::GetExtension($path)
    if ($ext -notin @(".rs", ".md", ".toml", ".yml", ".yaml", ".ps1")) {
        return
    }
    $count = @(Get-Content -LiteralPath $path).Count
    if ($count -gt 300) {
        $failed += "${path}: $count lines (limit 300)"
    }
    if ($ext -eq ".rs") {
        $lines = Get-Content -LiteralPath $path
        $fnStart = -1
        for ($i = 0; $i -lt $lines.Count; $i++) {
            if ($lines[$i] -match '^\s*(pub(\([^)]*\))?\s+)?((async|const)\s+)?fn\s') {
                if ($fnStart -ge 0) {
                    $len = $i - $fnStart
                    if ($len -gt 100) {
                        $name = ($lines[$fnStart] -replace '.*fn\s+', '' -replace '\(.*', '')
                        $failed += "${path}: fn $name is $len lines (limit 100)"
                    }
                }
                $fnStart = $i
            }
        }
        if ($fnStart -ge 0) {
            $len = $lines.Count - $fnStart
            if ($len -gt 100) {
                $name = ($lines[$fnStart] -replace '.*fn\s+', '' -replace '\(.*', '')
                $failed += "${path}: fn $name is $len lines (limit 100)"
            }
        }
    }
}

if ($failed.Count -gt 0) {
    $failed | ForEach-Object { Write-Error $_ }
    throw "Architecture budget failed ($($failed.Count) issues)"
}

Write-Output "All tracked files are <= 300 lines and Rust functions are <= 100 lines."
