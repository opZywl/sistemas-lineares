# Arquitetura do Sistemas Lineares

## Visão geral
O Sistemas Lineares tem três implementações da mesma matemática, que dão as mesmas respostas byte a byte:

| Parte | Pasta | Tecnologia | Papel |
| --- | --- | --- | --- |
| Núcleo | `native/core` | Rust e `num-bigint` | Frações exatas, leitura dos números, Gauss-Jordan e texto da resposta |
| App nativo | `native/app` | Rust, Win32, Direct2D e DirectWrite | O `SistemasLineares.exe` publicado nas releases |
| Versão HTML | `portable/` | JavaScript com BigInt, HTML e CSS | O `SistemasLineares.html`, que roda offline em qualquer sistema |
| Legado | `legado/csharp` | C#, WinForms e .NET Framework 4 | A versão 3, que hoje serve de referência de paridade |

## Estrutura do repositório
```
sistemas-lineares/
├── native/                    workspace Rust
│   ├── core/                  núcleo matemático (crate sistemas-lineares-core)
│   │   ├── src/lib.rs
│   │   └── tests/             solver, robustez, parser, racional, formato e paridade
│   ├── app/                   app Win32 (crate sistemas-lineares, binário SistemasLineares)
│   │   ├── src/               janela, desenho, layout, estado, instalação e preferências
│   │   ├── examples/          capturas de tela dos estados da interface
│   │   ├── build.rs           compila o app.rc com embed-resource
│   │   ├── app.rc             ícone, manifesto e versão do exe
│   │   └── app.manifest       Common Controls 6, DPI por monitor e asInvoker
│   └── Cargo.toml             membros, versão e perfil de release
├── portable/                  fontes da versão HTML
├── tools/build-portable.cjs   gera o SistemasLineares.html
├── tests/PortableTests.cjs    testes da versão HTML
├── legado/csharp/             versão 3 em C#, referência de paridade
├── assets/                    Space Grotesk, ícones SVG e app.ico
├── docs/                      guias e imagens do README
├── .github/                   workflows Build, Lint e Release e seus scripts
├── SistemasLineares.html      versão HTML gerada
├── abrir.cmd                  compila, testa e abre (modos html, instalar e desinstalar)
├── testar-tudo.cmd            validação completa com paridade
└── rust-toolchain.toml        Rust 1.99.0 com rustfmt e Clippy
```

## Núcleo (`native/core`)
A API pública é pequena e estável, e o app usa só ela:

- **`Rational`** guarda numerador e denominador como `BigInt`, sempre reduzidos e com o denominador positivo. Soma, subtração, multiplicação e divisão são exatas.
- **`Rational::parse`** aplica as regras de entrada: ignora espaços nas pontas, aceita vírgula ou ponto decimal, o sinal de menos tipográfico (`−`), uma fração `a/b` em que cada lado pode ter decimais e expoente, e notação científica. Os limites são 100 caracteres, contados em UTF-16 como no C#, e expoente entre -1000 e 1000. Os erros (`ParseError`) trazem as mesmas mensagens do C#.
- **`solve`** recebe a matriz aumentada n × (n + 1), com n = 2 ou 3, e aplica Gauss-Jordan. Em cada coluna, o pivô é a primeira linha não nula a partir da linha atual; essa linha sobe, é dividida pelo pivô e zera a coluna nas demais. Como a aritmética é exata, não há pivoteamento por magnitude. Uma linha 0 = c com c ≠ 0 dá `SolutionKind::None`, pivô em todas as colunas dá `Unique` e o resto dá `Infinite`. A `Solution` guarda a matriz reduzida e as colunas de pivô.
- **`format`** escreve a resposta exatamente como o C#, com `\r\n`: o valor de cada variável ou a forma geral com os parâmetros `t1`, `t2`, …

## App nativo (`native/app`)
| Módulo | Responsabilidade |
| --- | --- |
| `model` | Estado da tela e transições (resolver, exemplo, limpar, copiar, trocar 2×2 e 3×3), sem Win32 e testável sem janela |
| `layout` | Função pura que recebe a escala de DPI, a área cliente e a dimensão e devolve os retângulos de todos os elementos |
| `theme` | Cores, fontes e tamanhos |
| `gfx` | Direct2D e DirectWrite: formas, textos e ícones, sem depender de GPU |
| `fonts` | Space Grotesk embutida, carregada da memória pelo DirectWrite e pelo GDI (`AddFontMemResourceEx`) para os campos |
| `icons` | Desenha os SVGs de `assets/` (linhas, polilinhas, polígonos, retângulos e círculos) |
| `window` | Janela Win32, campos EDIT nativos, botões owner-draw, dicas, rolagem, alto contraste, acessibilidade e mensagens do sistema |
| `settings` | Guarda e restaura a posição, o tamanho, o estado maximizado e a dimensão da janela |
| `install` | Linha de comando, instalação e desinstalação por usuário |
| `registry` | Leitura e escrita no registro do usuário (`HKCU`) |
| `clipboard` | Copia a resposta como texto Unicode |

- **Tudo dentro do exe:** fontes e ícones entram com `include_bytes!`. O app não lê nenhum arquivo em execução.
- **Recursos do Windows:** o `build.rs` usa o `embed-resource` para compilar o `app.rc`, com o ícone, o manifesto (Common Controls 6, DPI PerMonitorV2 e `asInvoker`) e as informações de versão.
- **Controles nativos:** os campos são EDIT do Windows, então IME, menu de contexto, Ctrl+A/C/V/Z e leitores de tela funcionam como em qualquer app.
- **DPI por monitor:** ao mudar de monitor, o app recria fontes e recursos e refaz o layout com o retângulo sugerido pelo Windows.
- **Alto contraste:** com o alto contraste ligado, tudo é desenhado com as cores do sistema, e o app reage à troca de tema sem reiniciar.
- **Acessibilidade:** os campos têm nomes acessíveis, como "Equação 1, coeficiente de x", e o resultado e os erros são anunciados por leitores de tela como o Narrador e o NVDA.
- **Janela lembrada:** posição, tamanho, estado maximizado e dimensão ficam em `HKCU\Software\Lucas Lima\Sistemas Lineares`. A posição só é restaurada se ainda cair em um monitor conectado.
- **Perfil de release:** `opt-level = 3`, LTO, um codegen unit, `panic = "abort"` e `strip`.

### Linha de comando
| Argumento | O que faz | Código de saída |
| --- | --- | --- |
| nenhum | Abre o app | 0 |
| `--instalar` | Copia o exe para `%LOCALAPPDATA%\Programs\Sistemas Lineares`, cria o atalho no menu Iniciar e a entrada em Aplicativos instalados | 0 instalado, 1 erro |
| `--desinstalar` | Pede confirmação e remove a pasta, o atalho, a entrada e as preferências | 0 removido ou nada instalado, 2 cancelado, 1 erro |
| `--silencioso` | Junto com `--instalar` ou `--desinstalar`, dispensa perguntas e mensagens | igual ao do comando |

Argumentos desconhecidos são ignorados e o app abre normalmente.

## Versão HTML (`portable/`)
- **`solver.js`** implementa o mesmo algoritmo com BigInt e exporta `Rational`, `solve` e `format`, que os testes em Node também usam.
- **`app.js`**, **`styles.css`** e **`shell.html`** formam a interface.
- **`tools/build-portable.cjs`** junta tudo em um único `SistemasLineares.html`: o CSS com a fonte em base64, os ícones como `<symbol>` SVG e os scripts inline. O arquivo gerado fica no git, e o workflow Lint confere que ele corresponde às fontes.

## Legado C# (`legado/csharp`)
A versão 3 em C#/WinForms continua compilável com `legado\csharp\compilar.cmd`, mas não é publicada. O `testar-tudo.cmd` compila o `Solver.cs` e o `RobustnessTests.cs` com o `csc.exe` do .NET Framework 4 para gerar as respostas de referência.

## Testes e paridade
| Onde | O que verifica |
| --- | --- |
| `native/core/tests/solver.rs` | Porte do `SolverTests.cs`: 3.421 verificações, com os 1.000 sistemas gerados por `new Random(19)`, e o `System.Random` do .NET portado, conferido em 52.100 valores |
| `native/core/tests/robustez.rs` | Porte do `RobustnessTests.cs`: 76.674 sistemas e 1.703.886 verificações independentes (posto por menores, Cramer e substituição). Grava as respostas quando `SL_FIXTURES_SAIDA` está definida |
| `native/core/tests/parser.rs` | Casos de borda e limites de comprimento e expoente iguais aos do C#. O diferencial completo roda com `SL_PARSER_CSHARP` |
| `native/core/tests/racional.rs` | Normalização de sinal, mdc, zero, exibição e operações |
| `native/core/tests/formato.rs` | Os 7 exemplos da APRESENTACAO.md com o texto exato e, em sistemas variados, a classificação, os pivôs, a matriz reduzida e o texto comparados ao C# |
| `native/core/tests/paridade_*.rs` | Comparações opcionais com saídas do C#, ativadas por `SL_FIXTURES_CSHARP` e `SL_SOLVER_CSHARP` |
| `native/app` | Transições do estado, layout em várias escalas de DPI e testes de integração com a janela real |
| `tests/PortableTests.cjs` | As 76.674 respostas do JavaScript iguais às do Rust, a leitura dos números, IDs únicos e o HTML completo |

### Como a paridade é verificada
1. O `cargo test --release` roda com `SL_FIXTURES_SAIDA=tests\fixtures.tsv`, e o teste de robustez grava a resposta de cada um dos 76.674 sistemas.
2. O `csc.exe` compila a auditoria do C# legado, que grava `tests\fixtures-csharp.tsv` com os mesmos sistemas. O `fc /b` compara os dois arquivos byte a byte.
3. O `node tools/build-portable.cjs` gera o HTML, e o `node tests/PortableTests.cjs tests\fixtures.tsv` confere que o JavaScript dá as mesmas respostas.
4. Os arquivos temporários são apagados, e o script sai com 0 quando tudo passa ou 1 quando algo falha.

Cada linha das respostas tem o formato `n|entradas|tipo|resposta em base64`, em UTF-8. O Rust e o C# geram arquivos idênticos, com 8.052.533 bytes e SHA-256 `1cc866ab0133fee62657221dac933f5a00601b69fb40d3206018272832673fea`, e o JavaScript confere cada linha desse arquivo.

## Integração contínua
| Workflow | Quando roda | O que faz |
| --- | --- | --- |
| Build | Push e pull request na `main` | Roda o `testar-tudo.cmd`, compila com `cargo build --release --locked`, calcula o SHA-256 e guarda o exe, o HTML e o `SHA256SUMS.txt` como artefato |
| Lint | Push e pull request na `main` | Confere o rustfmt e o Clippy sem avisos e se o `SistemasLineares.html` commitado é igual ao gerado |
| Release | Tag `vX.Y.Z` | Confere a versão e o CHANGELOG, roda a validação, compila e publica a release com o exe, o HTML, o `SHA256SUMS.txt` e as notas do CHANGELOG |

A toolchain dos workflows vem do `rust-toolchain.toml`, e os scripts de apoio ficam em `.github/scripts`.

## Decisões
- **Frações exatas em vez de ponto flutuante:** em um sistema quase singular, com coeficientes como `1e-50`, o ponto flutuante confunde valores pequenos com zero e classifica errado. Com frações, a classificação é sempre exata.
- **Win32 e Direct2D em vez de WebView ou .NET:** um único exe, sem runtime para instalar, com controles nativos e desenho nítido em qualquer DPI.
- **C# mantido como legado:** a versão anterior virou o oráculo de paridade, e qualquer divergência do Rust aparece no `testar-tudo.cmd`.
- **HTML gerado e versionado:** quem não pode rodar o exe abre o HTML direto do repositório, e o Lint garante que ele está em dia.
