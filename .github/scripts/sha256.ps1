param(
    [string]$Executavel = 'native/target/release/SistemasLineares.exe',
    [string]$Html = 'SistemasLineares.html',
    [string]$Destino = 'native/target/dist'
)

$ErrorActionPreference = 'Stop'
$utf8 = New-Object System.Text.UTF8Encoding $false
$cultura = [System.Globalization.CultureInfo]::GetCultureInfo('pt-BR')

function Get-CaminhoCompleto([string]$Caminho) {
    $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Caminho)
}

function Format-Tamanho([long]$Bytes) {
    if ($Bytes -ge 1MB) {
        return [string]::Format($cultura, '{0:N1} MB', $Bytes / 1MB)
    }
    return [string]::Format($cultura, '{0:N1} KB', $Bytes / 1KB)
}

$origens = @($Executavel, $Html)
$faltando = @($origens | Where-Object { -not (Test-Path -LiteralPath $_ -PathType Leaf) })
if ($faltando.Count -gt 0) {
    throw "Arquivo não encontrado: $($faltando -join ', '). Compile o exe e gere o HTML antes de calcular o SHA-256."
}

$pasta = Get-CaminhoCompleto $Destino
New-Item -ItemType Directory -Force -Path $pasta | Out-Null

$arquivos = foreach ($origem in $origens) {
    $de = Get-CaminhoCompleto $origem
    $para = Join-Path $pasta (Split-Path -Leaf $de)
    if ($de -ne $para) {
        Copy-Item -LiteralPath $de -Destination $para -Force
    }
    Get-Item -LiteralPath $para
}

$linhas = foreach ($arquivo in $arquivos) {
    [pscustomobject]@{
        Nome = $arquivo.Name
        Tamanho = Format-Tamanho $arquivo.Length
        Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $arquivo.FullName).Hash.ToLowerInvariant()
    }
}

$somas = ($linhas | ForEach-Object { "$($_.Hash)  $($_.Nome)" }) -join "`n"
[System.IO.File]::WriteAllText((Join-Path $pasta 'SHA256SUMS.txt'), "$somas`n", $utf8)

foreach ($linha in $linhas) {
    Write-Output "$($linha.Hash)  $($linha.Nome)"
    if ($env:GITHUB_ACTIONS -eq 'true') {
        Write-Output "::notice title=SHA-256 $($linha.Nome)::$($linha.Hash)"
    }
}

$markdown = @(
    '### SHA-256'
    ''
    '| Arquivo | Tamanho | SHA-256 |'
    '| --- | --- | --- |'
)
$markdown += $linhas | ForEach-Object { '| `{0}` | {1} | `{2}` |' -f $_.Nome, $_.Tamanho, $_.Hash }
$markdown += @(
    ''
    'Para conferir um download no PowerShell: `Get-FileHash .\SistemasLineares.exe -Algorithm SHA256`'
)
$texto = ($markdown -join "`n") + "`n"
[System.IO.File]::WriteAllText((Join-Path $pasta 'SHA256SUMS.md'), $texto, $utf8)

if ($env:GITHUB_STEP_SUMMARY) {
    [System.IO.File]::AppendAllText($env:GITHUB_STEP_SUMMARY, $texto, $utf8)
}
