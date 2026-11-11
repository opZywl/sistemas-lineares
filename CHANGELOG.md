# Changelog

Todas as mudanças relevantes do Sistemas Lineares ficam registradas neste arquivo.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o projeto adota o [Versionamento Semântico](https://semver.org/lang/pt-BR/spec/v2.0.0.html).

## [Não lançado]

## [4.0.0] - 2026-11-11

### Adicionado
- **App nativo em Rust**
  - Um único `SistemasLineares.exe` com Win32, Direct2D e DirectWrite: sem .NET, sem WebView e sem console.
  - O mesmo visual e o mesmo comportamento da versão 3, com a Space Grotesk e os ícones dentro do exe.
  - DPI por monitor (Per-Monitor V2), funcionamento sem GPU e abertura sem clarão branco.
  - Alto contraste do Windows com as cores do sistema, atualizado sem reiniciar o app.
  - Leitor de tela: nomes acessíveis nos campos e anúncio do resultado e dos erros.
  - Janela lembrada entre aberturas: posição, tamanho, estado maximizado e dimensão 2×2 ou 3×3.
  - Instalação por usuário, sem administrador, com `--instalar` e `--desinstalar`: atalho no menu Iniciar e entrada em Aplicativos instalados.
- **Núcleo matemático em Rust** (`native/core`)
  - Porte do `Solver.cs` com respostas idênticas byte a byte às do C# nos 76.674 sistemas auditados.
  - Porte dos testes do C#, com os mesmos números: 3.421 verificações e 1.703.886 verificações independentes.
  - Porte do `System.Random` do .NET Framework, conferido em 52.100 valores, para gerar os mesmos sistemas aleatórios.
  - Leitura dos números idêntica à do C# em 991.677 entradas adversariais, e respostas idênticas em 60.000 sistemas racionais aleatórios.
- **Crédito com RA:** "Feito por Lucas Lima · RA 185964" no app e na versão HTML.
- **Ferramentas**
  - `abrir.cmd` compila, testa e abre o app, com os modos `html`, `instalar` e `desinstalar`.
  - `testar-tudo.cmd` roda os testes do Rust, a paridade byte a byte com o C# e os testes da versão HTML.
  - Workflows Build, Lint e Release no GitHub Actions; cada release traz o exe, o HTML e o SHA-256.
- **Documentação:** README, guias em `docs/`, `BUILD.md`, o CHANGELOG e a licença GPL-3.0.

### Alterado
- A versão C# saiu da raiz para `legado/csharp/` e passou a servir só de referência de paridade.
- O `SistemasLineares.exe` deixou de ficar no repositório e passou a ser publicado nas releases.

### Corrigido
- O subtítulo da versão HTML não fica mais grudado em telas estreitas.

### Removido
- A exigência do .NET Framework para usar o app.

## [3.0.0] - 2026-11-06

### Adicionado
- App para Windows em C#/WinForms (.NET Framework 4), com tema escuro lilás e a Space Grotesk e os ícones SVG dentro do exe.
- Versão HTML portátil (`SistemasLineares.html`), com o cálculo em JavaScript/BigInt e funcionamento offline.
- Frações exatas com `BigInteger` e eliminação de Gauss-Jordan: solução única, infinitas soluções com parâmetros livres ou sistema sem solução.
- Entrada de inteiros, decimais com vírgula ou ponto, frações e notação científica, com aviso no campo inválido.
- Testes: 3.421 verificações, auditoria de 76.674 sistemas com 1.703.886 verificações independentes e paridade entre C# e JavaScript.

[Não lançado]: https://github.com/opZywl/sistemas-lineares/compare/v4.0.0...HEAD
[4.0.0]: https://github.com/opZywl/sistemas-lineares/releases/tag/v4.0.0
[3.0.0]: https://github.com/opZywl/sistemas-lineares/tree/6b4c16b2d090f8de241b6d299185641a07aa95f6
