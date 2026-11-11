param(
    [Parameter(Mandatory = $true)][string]$Versao,
    [string]$Changelog = 'CHANGELOG.md',
    [string]$Cargo = 'native/Cargo.toml',
    [string]$Recurso = 'native/app/app.rc',
    [string]$Manifesto = 'native/app/app.manifest',
    [string]$Sha256 = '',
    [string]$Saida = ''
)

$ErrorActionPreference = 'Stop'
$utf8 = New-Object System.Text.UTF8Encoding $false

function Get-CaminhoCompleto([string]$Caminho) {
    $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Caminho)
}

function Read-Texto([string]$Caminho) {
    if (-not (Test-Path -LiteralPath $Caminho -PathType Leaf)) {
        throw "Arquivo não encontrado: $Caminho"
    }
    [System.IO.File]::ReadAllText((Get-CaminhoCompleto $Caminho), $utf8)
}

$Versao = $Versao -replace '^refs/tags/', '' -replace '^v', ''
if ($Versao -notmatch '^\d+\.\d+\.\d+$') {
    throw "Versão inválida: '$Versao'. A tag precisa ter o formato vX.Y.Z."
}

$pacote = [regex]::Match((Read-Texto $Cargo), '(?ms)^\[workspace\.package\][ \t]*\r?$(.*?)(?=^\[|\z)')
$versaoCargo = [regex]::Match($pacote.Groups[1].Value, '(?m)^[ \t]*version[ \t]*=[ \t]*"([^"]+)"').Groups[1].Value
if ($versaoCargo -ne $Versao) {
    throw "A tag é v$Versao, mas o [workspace.package] de $Cargo está com version = '$versaoCargo'."
}

if (Test-Path -LiteralPath $Recurso -PathType Leaf) {
    $rc = Read-Texto $Recurso
    $numerica = ($Versao -replace '\.', ',') + ',0'
    foreach ($campo in 'FILEVERSION', 'PRODUCTVERSION') {
        $achado = [regex]::Match($rc, '(?m)^[ \t]*' + $campo + '[ \t]+([0-9, \t]+?)[ \t]*\r?$')
        if ($achado.Success -and ($achado.Groups[1].Value -replace '\s', '') -ne $numerica) {
            throw "O $campo de $Recurso não corresponde à versão $Versao (esperado $numerica)."
        }
    }
    foreach ($achado in [regex]::Matches($rc, 'VALUE[ \t]+"(FileVersion|ProductVersion)"[ \t]*,[ \t]*"([^"]*)"')) {
        if ($achado.Groups[2].Value -ne "$Versao.0") {
            throw "O $($achado.Groups[1].Value) de $Recurso é '$($achado.Groups[2].Value)', mas a versão é $Versao.0."
        }
    }
}

if (Test-Path -LiteralPath $Manifesto -PathType Leaf) {
    $identidade = [regex]::Match((Read-Texto $Manifesto), '<assemblyIdentity\b[^>]*?\bversion="([^"]*)"')
    if ($identidade.Success -and $identidade.Groups[1].Value -ne "$Versao.0") {
        throw "O assemblyIdentity de $Manifesto está com version='$($identidade.Groups[1].Value)', mas a versão é $Versao.0."
    }
}

$linhas = (Read-Texto $Changelog) -split '\r?\n'
$cabecalho = '^##[ \t]+\[' + [regex]::Escape($Versao) + '\]'
$inicio = -1
for ($i = 0; $i -lt $linhas.Length; $i++) {
    if ($linhas[$i] -match $cabecalho) {
        $inicio = $i
        break
    }
}
if ($inicio -lt 0) {
    throw "O $Changelog não tem a seção ## [$Versao]. Registre as mudanças da versão antes de criar a tag."
}

$fim = $linhas.Length
for ($i = $inicio + 1; $i -lt $linhas.Length; $i++) {
    if ($linhas[$i] -match '^##[ \t]' -or $linhas[$i] -match '^\[[^\]]+\]:[ \t]' -or $linhas[$i] -match '^-{3,}[ \t]*$') {
        $fim = $i
        break
    }
}

$notas = [System.Collections.Generic.List[string]]::new()
for ($i = $inicio + 1; $i -lt $fim; $i++) {
    $notas.Add($linhas[$i])
}
while ($notas.Count -gt 0 -and [string]::IsNullOrWhiteSpace($notas[0])) {
    $notas.RemoveAt(0)
}
while ($notas.Count -gt 0 -and [string]::IsNullOrWhiteSpace($notas[$notas.Count - 1])) {
    $notas.RemoveAt($notas.Count - 1)
}
if ($notas.Count -eq 0) {
    throw "A seção ## [$Versao] do $Changelog está vazia."
}

if ($Sha256) {
    $notas.Add('')
    $notas.Add((Read-Texto $Sha256).TrimEnd())
}

if ($env:GITHUB_SERVER_URL -and $env:GITHUB_REPOSITORY) {
    $base = "$env:GITHUB_SERVER_URL/$env:GITHUB_REPOSITORY/blob/v$Versao"
    $notas.Add('')
    $notas.Add("[Como instalar]($base/docs/INSTALANDO.md) · [Histórico completo]($base/CHANGELOG.md)")
}

$texto = ($notas -join "`n") + "`n"
if ($Saida) {
    $arquivo = Get-CaminhoCompleto $Saida
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $arquivo) | Out-Null
    [System.IO.File]::WriteAllText($arquivo, $texto, $utf8)
}
Write-Output $texto
