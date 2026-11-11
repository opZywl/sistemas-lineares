$ErrorActionPreference = 'Stop'

$arquivo = @('rust-toolchain.toml', 'native/rust-toolchain.toml') |
    Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
    Select-Object -First 1
if (-not $arquivo) {
    throw 'Nenhum rust-toolchain.toml encontrado na raiz nem em native/.'
}

$canal = Select-String -LiteralPath $arquivo -Pattern '^\s*channel\s*=\s*["'']([^"'']+)["'']' |
    Select-Object -First 1 |
    ForEach-Object { $_.Matches[0].Groups[1].Value }
if (-not $canal) {
    throw "O $arquivo não define o channel da toolchain."
}

Write-Output "Toolchain fixada em ${arquivo}: $canal"
if ($env:GITHUB_OUTPUT) {
    [System.IO.File]::AppendAllText($env:GITHUB_OUTPUT, "canal=$canal`n", (New-Object System.Text.UTF8Encoding $false))
}
