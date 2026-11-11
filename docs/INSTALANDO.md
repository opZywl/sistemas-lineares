# Sistemas Lineares
![Plataforma: Windows](https://img.shields.io/badge/plataforma-Windows-CEA9F3?style=flat-square&labelColor=111010)\
Resolve sistemas lineares 2×2 e 3×3 com cálculo exato, em um app nativo para Windows.

Site: [lucas-lima.xyz](https://lucas-lima.xyz)\
Última versão: [Releases](https://github.com/opZywl/sistemas-lineares/releases/latest)\
Autor: Lucas Lima · RA 185964

## Como instalar o Sistemas Lineares?
O app roda no Windows 10 e 11 de 64 bits. Não precisa de .NET, de navegador nem de internet.

- **Passo 1:** Baixe o `SistemasLineares.exe` da [última release](https://github.com/opZywl/sistemas-lineares/releases/latest).
- **Passo 2:** Abra o arquivo com dois cliques. Ele é portátil e roda da pasta onde estiver.
- **Passo 3 (opcional):** Instale para ter o atalho no menu Iniciar, como explicado abaixo.

## Aviso do SmartScreen
O executável não tem assinatura digital. Por isso o navegador pode avisar que o arquivo não é baixado com frequência, e o Windows pode mostrar **O Windows protegeu o computador** na primeira execução. Nesse caso, clique em **Mais informações** e depois em **Executar assim mesmo**. Se quiser, confira antes o SHA-256 do arquivo, como explicado abaixo.

## Instalar e desinstalar
A instalação é por usuário e não pede administrador. Ela copia o app para `%LOCALAPPDATA%\Programs\Sistemas Lineares`, cria o atalho **Sistemas Lineares** no menu Iniciar e registra o app em **Configurações > Aplicativos > Aplicativos instalados**.

- **Instalar:** no PowerShell, na pasta do download, rode `.\SistemasLineares.exe --instalar`. Com o repositório clonado, rode `abrir.cmd instalar`.
- **Desinstalar:** em **Aplicativos instalados**, escolha **Sistemas Lineares** e depois **Desinstalar**. `.\SistemasLineares.exe --desinstalar` e `abrir.cmd desinstalar` fazem o mesmo.

Instalar de novo por cima atualiza a cópia instalada. A desinstalação remove a pasta, o atalho, a entrada em Aplicativos instalados e as preferências salvas.

## Usando
1. Escolha **2 × 2** ou **3 × 3**.
2. Preencha o coeficiente de cada variável e o termo independente, à direita do sinal de igual. Use **0** para a variável que não aparece na equação.
3. Clique em **Resolver sistema** ou pressione **Enter**. **Tab** passa para o próximo campo.
4. Clique em **Copiar resultado** para copiar a resposta completa, inclusive valores longos.

**Usar exemplo** preenche um sistema pronto, e **Limpar** apaga os campos.

### Formatos aceitos
Inteiros (`-2`), decimais com vírgula ou ponto (`1,5` ou `1.5`), frações (`3/4`, inclusive `2,5/0,5`) e notação científica (`1e-6`). Cada campo aceita até 100 caracteres e expoentes entre -1000 e 1000. Não use separador de milhar, e expressões como `√2` não são aceitas. Campo vazio, denominador zero e formato inválido são apontados no próprio campo.

## Versão HTML
O `SistemasLineares.html`, que está nas releases e na raiz do repositório, faz o mesmo cálculo no navegador. Abra com dois cliques em um navegador atual (Chrome, Edge, Firefox ou Safari) no Windows, macOS ou Linux. Ele funciona offline, porque o cálculo, os estilos, os ícones e a fonte estão dentro do arquivo. Com o repositório clonado, `abrir.cmd html` abre a versão HTML no Chrome.

## Conferindo o SHA-256
Cada release traz o `SHA256SUMS.txt` e a mesma tabela nas notas. No PowerShell:

```powershell
Get-FileHash .\SistemasLineares.exe -Algorithm SHA256
```

No Prompt de Comando, use `certutil -hashfile SistemasLineares.exe SHA256`. O valor tem que ser igual ao da release; maiúsculas e minúsculas não fazem diferença.

Com o `SHA256SUMS.txt` na mesma pasta, este comando do PowerShell confere todos os arquivos baixados e mostra `OK` ou `DIFERENTE` para cada um:

```powershell
Get-Content .\SHA256SUMS.txt | ForEach-Object { $hash, $nome = $_ -split '\s+', 2; if (Test-Path $nome) { "${nome}: " + $(if ((Get-FileHash $nome -Algorithm SHA256).Hash -eq $hash) { 'OK' } else { 'DIFERENTE' }) } }
```

## Onde o Sistemas Lineares guarda dados
O app guarda só a posição e o tamanho da janela, se ela estava maximizada e a dimensão 2×2 ou 3×3, em `HKCU\Software\Lucas Lima\Sistemas Lineares`. A cópia instalada fica em `%LOCALAPPDATA%\Programs\Sistemas Lineares`. Nada é enviado pela internet.
