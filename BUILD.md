# Compilando o Sistemas Lineares

A toolchain Rust 1.99.0 fica fixada no `rust-toolchain.toml`, e o [rustup](https://rustup.rs) a instala na primeira compilação. O Node.js é opcional e só serve para a versão HTML.

## Windows
Você precisa do Windows 10 ou 11 de 64 bits, do rustup e do Visual Studio Build Tools 2022 ou mais novo, com C++.

### O jeito curto
Rode `abrir.cmd` na raiz do repositório. Ele confere o Rust, compila, roda os testes, atualiza o `SistemasLineares.exe` da raiz e abre o app.

### Na mão
```
cd native
cargo build --release --locked
```
O executável fica em `native\target\release\SistemasLineares.exe`. Rode o `cargo` dentro de `native\` para usar a toolchain fixada. Sem `-p`, o Cargo compila só os crates do Windows.

Os testes do Rust rodam com `cargo test --release`, também dentro de `native\`. A validação completa, com a paridade com o C#, a versão HTML e a interface HTML no Chrome, é o `testar-tudo.cmd` na raiz.

### O que vai dentro do exe
As fontes Space Grotesk e os ícones SVG de `assets\` entram no exe na compilação. O `native\app\app.rc` adiciona o ícone `assets\app.ico`, o manifesto e as informações de versão. O app não lê nenhum arquivo do disco para funcionar.

## Linux
Você precisa de uma distribuição de 64 bits, do rustup, do compilador C e do `pkg-config`.

### O jeito curto
Rode `./abrir.sh` na raiz do repositório. Ele confere o Rust, compila, roda os testes e abre o app.

### Na mão
```
cd native
cargo build --release --locked -p sistemas-lineares-linux
```
O binário fica em `native/target/release/sistemas-lineares`. Os testes rodam com `cargo test --release -p sistemas-lineares-core -p sistemas-lineares-ui -p sistemas-lineares-linux`. A Space Grotesk, os ícones e os ícones do atalho entram no binário, que só depende das bibliotecas gráficas do sistema (OpenGL, X11 ou Wayland), carregadas na hora de abrir.

## Versão HTML e legado
A versão HTML sai de `node tools/build-portable.cjs`, que junta `portable/` e `assets/` no `SistemasLineares.html`. A versão C# de referência compila com `legado\csharp\compilar.cmd`.

## Capturas e demonstração
- `cargo run -p sistemas-lineares --example capturas -- <pasta>` salva as capturas de todos os estados do app Windows. Use `--gdi` para o desenho sem Direct2D e `--alto-contraste` para as cores do sistema.
- `cargo run -p sistemas-lineares-linux --example capturas -- <pasta>` faz o mesmo no Linux.
- `cargo run -p sistemas-lineares --example demo -- <pasta>` grava os quadros da demonstração e a lista `quadros.txt`. O GIF do README sai do FFmpeg, dentro da pasta dos quadros:

```
ffmpeg -f concat -safe 0 -i quadros.txt -vf "scale=960:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=128[p];[b][p]paletteuse=dither=none" -fps_mode vfr demo.gif
```

## Releases
Os binários não vão para o git. Uma tag `vX.Y.Z` dispara o workflow Release, que testa, compila e publica o `SistemasLineares.exe`, o `SistemasLineares.html`, o `SHA256SUMS.txt`, o binário Linux `sistemas-lineares` e o `SHA256SUMS-linux.txt`. O passo a passo está em [CONTRIBUINDO.md](docs/CONTRIBUINDO.md#publicando-uma-versão).
