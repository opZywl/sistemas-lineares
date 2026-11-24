# Sistemas Lineares
![Plataformas: Windows e Linux](https://img.shields.io/badge/plataforma-Windows%20%7C%20Linux-CEA9F3?style=flat-square&labelColor=111010)\
Resolve sistemas lineares 2×2 e 3×3 com cálculo exato, em apps nativos para Windows e Linux.

Site: [lucas-lima.xyz](https://lucas-lima.xyz)\
Última versão: [Releases](https://github.com/opZywl/sistemas-lineares/releases/latest)\
Autor: Lucas Lima · RA 185964

## Configurando o ambiente
Os apps nativos formam um workspace Rust em `native/`: o núcleo matemático, o modelo da tela compartilhado e um app para cada sistema. A versão HTML é gerada com Node.js, e a versão C# de referência compila com o compilador do .NET Framework 4, que já vem no Windows.

A toolchain fica fixada no `rust-toolchain.toml` (Rust 1.99.0 com rustfmt e Clippy), e o [rustup](https://rustup.rs) a instala na primeira compilação.

### No Windows
Você precisa de:
- Windows 10 ou 11 de 64 bits;
- [rustup](https://rustup.rs);
- Visual Studio Build Tools 2022 ou mais novo, com **Desenvolvimento para desktop com C++**, que traz o linker MSVC e o Windows SDK. O `rc.exe` do SDK coloca o ícone, o manifesto e a versão no exe;
- Node.js 18 ou mais novo, para gerar e testar a versão HTML (opcional);
- Google Chrome, para o teste da interface HTML (opcional);
- .NET Framework 4, que já vem no Windows, para a paridade com a versão C# (opcional).

1. Clone o repositório: `git clone https://github.com/opZywl/sistemas-lineares.git`
2. Entre na pasta: `cd sistemas-lineares`
3. Rode `abrir.cmd`. Ele confere o Rust, compila, roda os testes, atualiza o `SistemasLineares.exe` da raiz e abre o app.
4. Rode `testar-tudo.cmd` para a validação completa: testes do Rust, paridade com o C#, versão HTML e interface HTML no Chrome.

### Modos do abrir.cmd
| Comando | O que faz |
| --- | --- |
| `abrir.cmd` | Compila, testa e abre o app. Sem o Rust, abre o `SistemasLineares.exe` que estiver na raiz. |
| `abrir.cmd html` | Gera o `SistemasLineares.html`, se houver Node.js, e abre no Chrome. |
| `abrir.cmd instalar` | Compila, testa e instala no menu Iniciar. |
| `abrir.cmd desinstalar` | Remove a instalação. |
| `--no-pause` | Termina sem pausa no fim, para uso em scripts. Vale para qualquer modo. |

### No Linux
Você precisa de:
- uma distribuição de 64 bits com X11 ou Wayland;
- [rustup](https://rustup.rs);
- o compilador C e o `pkg-config` (no Ubuntu e no Debian: `sudo apt install build-essential pkg-config`);
- Node.js e Google Chrome, só para a versão HTML (opcional).

1. Clone o repositório e entre na pasta, como no Windows.
2. Rode `./abrir.sh`. Ele confere o Rust e o compilador C, compila, roda os testes e abre o app.

| Comando | O que faz |
| --- | --- |
| `./abrir.sh` | Compila, testa e abre o app. |
| `./abrir.sh html` | Gera o `SistemasLineares.html`, se houver Node.js, e abre no Chrome. |
| `./abrir.sh instalar` | Compila, testa e instala para o seu usuário, com atalho no menu de aplicativos. |
| `./abrir.sh desinstalar` | Remove a instalação. |

No WSL, com o repositório em `/mnt/c`, o `abrir.sh` compila em `~/.cache/sistemas-lineares/target`, porque compilar direto na pasta do Windows é bem mais lento. O app abre pelo WSLg.

### Comandos do Rust
Rode dentro de `native/`, para usar a toolchain fixada. Sem `-p`, o Cargo usa os crates do Windows (`core`, `ui` e `app`):

```
cargo build --release
cargo test --release
cargo fmt --all
cargo clippy --all-targets --locked -- -D warnings
```

No Linux, inclua o app Linux pelo nome do pacote:

```
cargo build --release -p sistemas-lineares-linux
cargo test --release -p sistemas-lineares-core -p sistemas-lineares-ui -p sistemas-lineares-linux
cargo clippy -p sistemas-lineares-linux --all-targets --locked -- -D warnings
```

### Problemas comuns
- **`link.exe` não encontrado:** instale o Visual Studio Build Tools com **Desenvolvimento para desktop com C++**.
- **`rc.exe` não encontrado:** adicione o Windows SDK pelo instalador do Visual Studio.
- **`cc` não encontrado no Linux:** instale o `build-essential` (ou o equivalente da sua distribuição).
- **Aviso de Node.js, Chrome ou .NET Framework no `testar-tudo.cmd`:** aquela etapa foi pulada. Instale a ferramenta para a validação completa.
- **O exe da raiz não atualiza:** feche o Sistemas Lineares antes de recompilar.

Mais detalhes de compilação estão em [BUILD.md](../BUILD.md).
