# Sistemas Lineares
![Plataforma: Windows](https://img.shields.io/badge/plataforma-Windows-CEA9F3?style=flat-square&labelColor=111010)\
Resolve sistemas lineares 2×2 e 3×3 com cálculo exato, em um app nativo para Windows.

Site: [lucas-lima.xyz](https://lucas-lima.xyz)\
Última versão: [Releases](https://github.com/opZywl/sistemas-lineares/releases/latest)\
Autor: Lucas Lima · RA 185964

## Configurando o ambiente
O app nativo é um workspace Rust em `native/`. A versão HTML é gerada com Node.js, e a versão C# de referência compila com o compilador do .NET Framework 4, que já vem no Windows.

Você precisa de:
- Windows 10 ou 11 de 64 bits;
- [rustup](https://rustup.rs). A toolchain fica fixada no `rust-toolchain.toml` (Rust 1.99.0 com rustfmt e Clippy) e o rustup a instala na primeira compilação;
- Visual Studio Build Tools 2022 ou mais novo, com **Desenvolvimento para desktop com C++**, que traz o linker MSVC e o Windows SDK. O `rc.exe` do SDK coloca o ícone, o manifesto e a versão no exe;
- Node.js 18 ou mais novo, para gerar e testar a versão HTML (opcional);
- .NET Framework 4, que já vem no Windows, para a paridade com a versão C# (opcional).

1. Clone o repositório: `git clone https://github.com/opZywl/sistemas-lineares.git`
2. Entre na pasta: `cd sistemas-lineares`
3. Rode `abrir.cmd`. Ele confere o Rust, compila, roda os testes, atualiza o `SistemasLineares.exe` da raiz e abre o app.
4. Rode `testar-tudo.cmd` para a validação completa: testes do Rust, paridade com o C# e versão HTML.

### Modos do abrir.cmd
| Comando | O que faz |
| --- | --- |
| `abrir.cmd` | Compila, testa e abre o app. Sem o Rust, abre o `SistemasLineares.exe` que estiver na raiz. |
| `abrir.cmd html` | Gera o `SistemasLineares.html`, se houver Node.js, e abre no Chrome. |
| `abrir.cmd instalar` | Compila, testa e instala no menu Iniciar. |
| `abrir.cmd desinstalar` | Remove a instalação. |
| `--no-pause` | Termina sem pausa no fim, para uso em scripts. Vale para qualquer modo. |

### Comandos do Rust
Rode dentro de `native/`, para usar a toolchain fixada:

```
cargo build --release
cargo test --release
cargo fmt --all
cargo clippy --all-targets --locked -- -D warnings
```

### Problemas comuns
- **`link.exe` não encontrado:** instale o Visual Studio Build Tools com **Desenvolvimento para desktop com C++**.
- **`rc.exe` não encontrado:** adicione o Windows SDK pelo instalador do Visual Studio.
- **Aviso de Node.js ou do .NET Framework no `testar-tudo.cmd`:** aquela etapa foi pulada. Instale a ferramenta para a validação completa.
- **O exe da raiz não atualiza:** feche o Sistemas Lineares antes de recompilar.

Mais detalhes de compilação estão em [BUILD.md](../BUILD.md).
