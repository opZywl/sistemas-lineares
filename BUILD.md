# Compilando o Sistemas Lineares

Você precisa do Windows 10 ou 11 de 64 bits, do [rustup](https://rustup.rs) e do Visual Studio Build Tools 2022 ou mais novo, com C++. A toolchain Rust 1.99.0 fica fixada no `rust-toolchain.toml` e o rustup a instala na primeira compilação. O Node.js é opcional e só serve para a versão HTML.

## O jeito curto
Rode `abrir.cmd` na raiz do repositório. Ele confere o Rust, compila, roda os testes, atualiza o `SistemasLineares.exe` da raiz e abre o app.

## Na mão
```
cd native
cargo build --release --locked
```
O executável fica em `native\target\release\SistemasLineares.exe`. Rode o `cargo` dentro de `native\` para usar a toolchain fixada.

Os testes do Rust rodam com `cargo test --release`, também dentro de `native\`. A validação completa, com a paridade com o C# e a versão HTML, é o `testar-tudo.cmd` na raiz.

A versão HTML sai de `node tools\build-portable.cjs`, que junta `portable\` e `assets\` no `SistemasLineares.html`. A versão C# de referência compila com `legado\csharp\compilar.cmd`.

## O que vai dentro do exe
As fontes Space Grotesk e os ícones SVG de `assets\` entram no exe na compilação. O `native\app\app.rc` adiciona o ícone `assets\app.ico`, o manifesto e as informações de versão. O app não lê nenhum arquivo do disco para funcionar.

## Releases
O exe não vai para o git. Uma tag `vX.Y.Z` dispara o workflow Release, que testa, compila e publica o `SistemasLineares.exe`, o `SistemasLineares.html` e o `SHA256SUMS.txt`. O passo a passo está em [CONTRIBUINDO.md](docs/CONTRIBUINDO.md#publicando-uma-versão).
