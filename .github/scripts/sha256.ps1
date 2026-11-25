param(
    [string]$Destino = 'native/target/dist',
    [string]$Executavel = '',
    [string]$Html = ''
)

$ErrorActionPreference = 'Stop'
$utf8 = New-Object System.Text.UTF8Encoding $false

function Get-CaminhoCompleto([string]$Caminho) {
    $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Caminho)
}

$sistemas = [ordered]@{
    'SistemasLineares-Windows.exe' = 'Windows 10 e 11 (64 bits)'
    'SistemasLineares-Linux' = 'Linux (64 bits)'
    'SistemasLineares-Navegador.html' = 'Qualquer sistema, no navegador'
}

$pasta = Get-CaminhoCompleto $Destino
New-Item -ItemType Directory -Force -Path $pasta | Out-Null

foreach ($copia in @(@($Executavel, 'SistemasLineares-Windows.exe'), @($Html, 'SistemasLineares-Navegador.html'))) {
    $origem, $nome = $copia
    if (-not $origem) {
        continue
    }
    if (-not (Test-Path -LiteralPath $origem -PathType Leaf)) {
        throw "Arquivo não encontrado: $origem. Compile o exe e gere o HTML antes de calcular o SHA-256."
    }
    Copy-Item -LiteralPath (Get-CaminhoCompleto $origem) -Destination (Join-Path $pasta $nome) -Force
}

$arquivos = @($sistemas.Keys | ForEach-Object { Join-Path $pasta $_ } | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | ForEach-Object { Get-Item -LiteralPath $_ })
if ($arquivos.Count -eq 0) {
    throw "Nenhum arquivo da release em ${pasta}: esperado $($sistemas.Keys -join ', ')."
}

$linhas = foreach ($arquivo in $arquivos) {
    [pscustomobject]@{
        Sistema = $sistemas[$arquivo.Name]
        Nome = $arquivo.Name
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
    '| Sistema | Arquivo | SHA-256 |'
    '| --- | --- | --- |'
)
$markdown += $linhas | ForEach-Object { '| {0} | `{1}` | `{2}` |' -f $_.Sistema, $_.Nome, $_.Hash }
$texto = ($markdown -join "`n") + "`n"
[System.IO.File]::WriteAllText((Join-Path $pasta 'SHA256SUMS.md'), $texto, $utf8)

if ($env:GITHUB_STEP_SUMMARY) {
    [System.IO.File]::AppendAllText($env:GITHUB_STEP_SUMMARY, $texto, $utf8)
}
