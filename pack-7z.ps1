$ErrorActionPreference = 'Stop'

$candidates = @(
    'C:\Program Files\7-Zip\7z.exe',
    'C:\Program Files (x86)\7-Zip\7z.exe'
)
$sevenZip = $candidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $sevenZip) {
    $inPath = Get-Command 7z -ErrorAction SilentlyContinue
    if ($inPath) { $sevenZip = $inPath.Source }
}
if (-not $sevenZip) {
    Write-Error '未找到 7-Zip，请安装后重试（https://www.7-zip.org）'
    exit 1
}

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$out = Join-Path $root 'folder-manager-src.7z'

if (Test-Path -LiteralPath $out) { Remove-Item -LiteralPath $out -Force }

Push-Location $root
try {
    & $sevenZip a -t7z -mx=7 $out * `
        '-xr!node_modules' `
        '-xr!src-tauri/target' `
        '-xr!src-tauri/gen' `
        '-xr!dist' `
        '-xr!.git' `
        '-xr!*.7z'
    if ($LASTEXITCODE -ne 0) { throw "7z 压缩失败（exit code $LASTEXITCODE）" }
} finally {
    Pop-Location
}

$size = (Get-Item -LiteralPath $out).Length
Write-Output ''
Write-Output ('完成: {0} ({1:N2} MB)' -f $out, ($size / 1MB))
