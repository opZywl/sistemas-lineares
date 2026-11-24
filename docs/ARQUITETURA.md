# Arquitetura do Sistemas Lineares

## Visão geral
O Sistemas Lineares tem três implementações da mesma matemática, que dão as mesmas respostas byte a byte, e dois apps nativos que compartilham o mesmo modelo da tela:

| Parte | Pasta | Tecnologia | Papel |
| --- | --- | --- | --- |
| Núcleo | `native/core` | Rust e `num-bigint` | Frações exatas, leitura dos números (com potências), Gauss-Jordan, texto da resposta e Modo pensar |
| Modelo da tela | `native/ui` | Rust | Estado da tela, layout, tema, quebra de texto e layout do Modo pensar, comuns aos dois apps nativos |
| App para Windows | `native/app` | Rust, Win32, Direct2D e DirectWrite | O `SistemasLineares.exe` publicado nas releases |
| App para Linux | `native/linux` | Rust, eframe, egui e AccessKit | O `sistemas-lineares` publicado nas releases |
| Versão HTML | `portable/` | JavaScript com BigInt, HTML e CSS | O `SistemasLineares.html`, que roda offline em qualquer sistema |
| Legado | `legado/csharp` | C#, WinForms e .NET Framework 4 | A versão 3, que hoje serve de referência de paridade |

## Estrutura do repositório
```
sistemas-lineares/
├── native/                    workspace Rust
│   ├── core/                  núcleo matemático (crate sistemas-lineares-core)
│   │   ├── src/               lib.rs, power.rs (potências) e steps.rs (Modo pensar)
│   │   └── tests/             solver, robustez, parser, potências, passos, racional, formato e paridade
│   ├── ui/                    modelo da tela, layout, tema e Modo pensar (crate sistemas-lineares-ui)
│   ├── app/                   app Win32 (crate sistemas-lineares, binário SistemasLineares)
│   │   ├── src/               janela, desenho, fontes, instalação e preferências
│   │   ├── examples/          capturas de tela dos estados e quadros da demonstração
│   │   ├── build.rs           compila o app.rc com embed-resource
│   │   ├── app.rc             ícone, manifesto e versão do exe
│   │   └── app.manifest       Common Controls 6, DPI por monitor e asInvoker
│   ├── linux/                 app Linux (crate sistemas-lineares-linux, binário sistemas-lineares)
│   └── Cargo.toml             membros, versão e perfil de release
├── portable/                  fontes da versão HTML
├── tools/                    build-portable.cjs gera o HTML; gerar-potencias.py gera o vetor de potências
├── tests/                     testes da versão HTML e vetor de potências compartilhado
├── legado/csharp/             versão 3 em C#, referência de paridade
├── assets/                    Space Grotesk, ícones SVG e app.ico
├── docs/                      guias e imagens do README
├── .github/                   workflows Build, Lint e Release e seus scripts
├── SistemasLineares.html      versão HTML gerada
├── abrir.cmd                  Windows: compila, testa e abre (modos html, instalar e desinstalar)
├── abrir.sh                   Linux: compila, testa e abre (mesmos modos)
├── testar-tudo.cmd            validação completa com paridade
└── rust-toolchain.toml        Rust 1.99.0 com rustfmt e Clippy
```

## Núcleo (`native/core`)
A API pública é pequena e estável, e os apps usam só ela:

- **`Rational`** guarda numerador e denominador como `BigInt`, sempre reduzidos e com o denominador positivo. Soma, subtração, multiplicação e divisão são exatas.
- **`Rational::parse`** aplica as regras de entrada: ignora espaços nas pontas, aceita vírgula ou ponto decimal, o sinal de menos tipográfico (`−`), uma fração `a/b` em que cada lado pode ter decimais e expoente, e notação científica. Os limites são 100 caracteres, contados em UTF-16 como no C#, e expoente entre -1000 e 1000. Sem potência, a leitura e as mensagens são as mesmas do C#.
- **Potências (`power`)** entram quando o texto tem `^`, `**`, parênteses, expoentes sobrescritos ou um sinal de multiplicação. A gramática aceita `2^3`, `2**3`, `10^-2`, `(1/2)^3` e `10²`; o sinal vale depois da potência (`-2² = −4`); potência de potência pede parênteses; o expoente é inteiro, de -1000 a 1000. Todo valor que passa por uma potência precisa ficar abaixo de 10^300 no numerador e no denominador, o que mantém a resposta instantânea mesmo com números densos. Erros próprios: `PowerExponent`, `PowerChain`, `PowerTooLarge`, `ZeroToZero`, `ZeroNegativePower`, `Parenthesis` e `Product`.
- **Resposta ao vivo:** `looks_incomplete` reconhece um número ainda sendo digitado (como `2^` ou `1/`), e `within_live_budget` limita a resolução automática a 8.000 bits somando numeradores e denominadores. Acima disso, a conta espera o Enter.
- **Prévia:** `preview_equation` escreve uma equação com `?` no lugar dos campos que faltam, e `abbreviate` encurta cada número com mais de 12 algarismos para os 4 primeiros, `…` e os 4 últimos.
- **`solve`** recebe a matriz aumentada n × (n + 1), com n = 2 ou 3, e aplica Gauss-Jordan. Em cada coluna, o pivô é a primeira linha não nula a partir da linha atual; essa linha sobe, é dividida pelo pivô e zera a coluna nas demais. Como a aritmética é exata, não há pivoteamento por magnitude. Uma linha 0 = c com c ≠ 0 dá `SolutionKind::None`, pivô em todas as colunas dá `Unique` e o resto dá `Infinite`. A `Solution` guarda a matriz reduzida e as colunas de pivô.
- **`format`** escreve a resposta exatamente como o C#, com `\r\n`: o valor de cada variável ou a forma geral com os parâmetros `t1`, `t2`, …
- **`walkthrough`** (módulo `steps`) monta o passo a passo do Modo pensar para um método (`Method::GaussJordan`, `Method::Cramer` ou `Method::Gauss`). Cada `Step` tem título, raciocínio, linhas de conta (`LineKind::Operation`, `Calculation` ou `Result`), uma `MatrixView` opcional com o pivô, as linhas e a coluna em destaque, e as operações de linha (`RowOp`) que os testes refazem. O Gauss-Jordan segue exatamente os pivôs do `solve`; o escalonamento resolve de baixo para cima com expressões nos parâmetros; a regra de Cramer usa a diagonal no 2×2 e Sarrus no 3×3 e, com D = 0, continua por Gauss-Jordan. `Walkthrough::text` gera o texto copiado, que também é a referência de paridade com o JavaScript.

## Modelo da tela (`native/ui`)
| Módulo | Responsabilidade |
| --- | --- |
| `model` | Estado da tela e transições, sem janela: edição, foco, pausa na digitação, saída do campo, Enter, setas, Resolver, exemplo, limpar, trocar 2×2 e 3×3, links do rodapé, copiar e Modo pensar. Também todos os textos fixos da interface |
| `layout` | Função pura que recebe a escala de DPI, a área cliente e a dimensão e devolve os retângulos de todos os elementos, inclusive a prévia, os trilhos dos seletores, os links e as teclas do rodapé |
| `theme` | Cores, fontes e tamanhos, inclusive a paleta de alto contraste com as cores do sistema |
| `thinking` | Layout do Modo pensar: quebra os textos, mede as matrizes e devolve os itens de desenho de cada passo |
| `text` | Quebra de linha e reticências, com poucas medidas mesmo em números de mil algarismos |
| `icons` | Lê os SVGs de `assets/` (linhas, polilinhas, polígonos, retângulos e círculos) |

O `model` é a especificação do comportamento, e os dois apps nativos só desenham o que ele diz:

- **Resposta ao vivo:** cada tecla chama `edit`. Com todos os campos válidos e dentro do orçamento, o resultado aparece na hora. Se já havia resultado e a edição deixou o sistema incompleto, o resultado anterior fica esmaecido até `idle` (700 ms sem digitar) ou `leave` (saída do campo).
- **Erros por campo:** um campo inválido revela o erro ao sair dele, no Enter ou na pausa, exceto quando ainda é um número sendo digitado no campo focado. O erro continua enquanto o campo seguir inválido e some na hora em que ele fica válido ou vazio.
- **Estados do card:** `Card::Result` (com ou sem esmaecimento) ou `Card::Waiting`, que pode ser `Empty`, `Missing`, `Invalid` ou `Large`, com a prévia das equações.
- **Cada tamanho guarda o próprio sistema:** o 2×2 e o 3×3 têm grades separadas, e Limpar apaga as duas.
- **Dica abaixo dos campos:** o erro do campo focado, os avisos, o valor lido de um campo com potência ("Este campo vale 8.") ou a explicação dos formatos.
- **Anúncios:** só os estados de resultado são anunciados. Na digitação, o anúncio sai depois da pausa e só quando o texto mudou; ações como Resolver, Enter, Usar exemplo e os links anunciam na hora.

## App para Windows (`native/app`)
| Módulo | Responsabilidade |
| --- | --- |
| `gfx` | Direct2D e DirectWrite: formas, anéis de foco, textos e ícones, sem depender de GPU, com desenho por GDI quando o Direct2D falha e cache das larguras de texto |
| `fonts` | Space Grotesk embutida, carregada da memória pelo DirectWrite e pelo GDI (`AddFontMemResourceEx`) para os campos |
| `window` | Janela Win32, campos EDIT nativos, botões e links owner-draw, temporizador da pausa, hover, dicas, rolagem, alto contraste, acessibilidade e mensagens do sistema |
| `settings` | Guarda e restaura a posição, o tamanho, o estado maximizado, a dimensão e o método do Modo pensar |
| `install` | Linha de comando, instalação e desinstalação por usuário |
| `registry` | Leitura e escrita no registro do usuário (`HKCU`) |
| `clipboard` | Copia a resposta e o passo a passo como texto Unicode |

- **Tudo dentro do exe:** fontes e ícones entram com `include_bytes!`. O app não lê nenhum arquivo em execução.
- **Recursos do Windows:** o `build.rs` usa o `embed-resource` para compilar o `app.rc`, com o ícone, o manifesto (Common Controls 6, DPI PerMonitorV2 e `asInvoker`) e as informações de versão.
- **Controles nativos:** os campos são EDIT do Windows, então IME, menu de contexto, Ctrl+A/C/V/Z e leitores de tela funcionam como em qualquer app. Ctrl+A e Ctrl+Backspace só agem com Ctrl sem Alt, porque AltGr é Ctrl+Alt.
- **DPI por monitor:** ao mudar de monitor, o app recria fontes e recursos e refaz o layout com o retângulo sugerido pelo Windows.
- **Alto contraste:** com o alto contraste ligado, tudo é desenhado com as cores do sistema, e o app reage à troca de tema sem reiniciar.
- **Acessibilidade:** os campos têm nomes como "Equação 1, coeficiente de x" e a descrição com os formatos aceitos e as teclas. Os seletores dizem se estão selecionados, os links do rodapé explicam o método, e o resultado e os erros são anunciados por regiões vivas para o Narrador e o NVDA.
- **Janela lembrada:** posição, tamanho, estado maximizado, dimensão e método ficam em `HKCU\Software\Lucas Lima\Sistemas Lineares`. A posição só é restaurada se ainda cair em um monitor conectado.
- **Perfil de release:** `opt-level = 3`, LTO, um codegen unit, `panic = "abort"` e `strip`.

### Linha de comando no Windows
| Argumento | O que faz | Código de saída |
| --- | --- | --- |
| nenhum | Abre o app | 0 |
| `--instalar` | Copia o exe para `%LOCALAPPDATA%\Programs\Sistemas Lineares`, cria o atalho no menu Iniciar e a entrada em Aplicativos instalados | 0 instalado, 1 erro |
| `--desinstalar` | Pede confirmação e remove a pasta, o atalho, a entrada e as preferências | 0 removido ou nada instalado, 2 cancelado, 1 erro |
| `--silencioso` | Junto com `--instalar` ou `--desinstalar`, dispensa perguntas e mensagens | igual ao do comando |

Argumentos desconhecidos são ignorados e o app abre normalmente.

## App para Linux (`native/linux`)
| Módulo | Responsabilidade |
| --- | --- |
| `app` | Janela eframe com o mesmo layout, o mesmo modelo e os mesmos estados do app para Windows, teclado, foco, links, dicas e Modo pensar |
| `paint` | Desenho com o egui: cards, trilhos, anéis de foco, textos com as mesmas margens do Windows e ícones |
| `fonts` | Space Grotesk embutida e métricas iguais às do Windows |
| `contrast` | Alto contraste pela opção `--alto-contraste`, pela variável de ambiente ou pela preferência do desktop (portal ou GNOME) |
| `install` | Instalação e desinstalação por usuário nas pastas XDG: o binário em `~/.local/bin`, o atalho `.desktop` e os ícones |
| `cli` | Linha de comando: `--instalar`, `--desinstalar`, `--silencioso`, `--alto-contraste`, `--versao` e `--ajuda` |

- **Acessibilidade:** o AccessKit publica a árvore de acessibilidade pelo AT-SPI, com os mesmos nomes e descrições do Windows, para o Orca.
- **Preferências:** tamanho e posição da janela, dimensão e método ficam em `~/.config/sistemas-lineares/preferencias.ron`.
- **Gráficos:** OpenGL pelo `glow`, com X11 ou Wayland. As bibliotecas do sistema são carregadas na hora de abrir.

## Versão HTML (`portable/`)
- **`solver.js`** implementa o mesmo algoritmo com BigInt e exporta `Rational`, `solve` e `format`, que os testes em Node também usam. É um porte fiel do núcleo, inclusive das potências, dos espaços Unicode aceitos, da prévia e do orçamento da resposta ao vivo. Também exporta `METHODS`, `walkthrough` e `walkthroughText`, o porte do Modo pensar, que gera o mesmo texto do Rust.
- **`app.js`**, **`styles.css`** e **`shell.html`** formam a interface. O `app.js` porta o comportamento do `native/ui/src/model.rs`: resposta ao vivo, esmaecimento, erros por campo, Enter e setas, links do rodapé e anúncios.
- **`tools/build-portable.cjs`** junta tudo em um único `SistemasLineares.html`: o CSS com a fonte em base64, os ícones como `<symbol>` SVG e os scripts inline. O arquivo gerado fica no git, e o workflow Lint confere que ele corresponde às fontes.

## Legado C# (`legado/csharp`)
A versão 3 em C#/WinForms continua compilável com `legado\csharp\compilar.cmd`, mas não é publicada. O `testar-tudo.cmd` compila o `Solver.cs` e o `RobustnessTests.cs` com o `csc.exe` do .NET Framework 4 para gerar as respostas de referência. O C# não tem potências; o diferencial de leitura com ele ignora as entradas com potência.

## Testes e paridade
| Onde | O que verifica |
| --- | --- |
| `native/core/tests/solver.rs` | Porte do `SolverTests.cs`: 3.421 verificações, com os 1.000 sistemas gerados por `new Random(19)`, e o `System.Random` do .NET portado, conferido em 52.100 valores |
| `native/core/tests/robustez.rs` | Porte do `RobustnessTests.cs`: 76.674 sistemas e 1.703.886 verificações independentes (posto por menores, Cramer e substituição). Grava as respostas quando `SL_FIXTURES_SAIDA` está definida |
| `native/core/tests/parser.rs` | Casos de borda e limites de comprimento e expoente iguais aos do C#. O diferencial completo roda com `SL_PARSER_CSHARP` |
| `native/core/tests/potencias.rs` | O vetor `tests/potencias.txt`, cujas respostas saem do gerador independente `tools/gerar-potencias.py` (frações do Python), as propriedades das potências em milhares de bases, os limites exatos de algarismos, a prévia e o pior caso de desempenho. Grava o corpus diferencial de 200.000 entradas quando `SL_POTENCIAS_SAIDA` está definida |
| `native/core/tests/racional.rs` | Normalização de sinal, mdc, zero, exibição e operações |
| `native/core/tests/passos.rs` | O Modo pensar nos três métodos: em todos os 15.625 sistemas 2×2 pequenos, 39.366 sistemas 3×3 e 3.000 sistemas com frações, refaz cada operação de linha, compara cada matriz mostrada, recalcula D, Dx, Dy e Dz pela definição e confere a resposta e a verificação. Grava o texto dos passos quando `SL_PASSOS_SAIDA` está definida |
| `native/core/tests/formato.rs` | Os 7 exemplos da APRESENTACAO.md com o texto exato e, em sistemas variados, a classificação, os pivôs, a matriz reduzida e o texto comparados ao C# |
| `native/core/tests/paridade_*.rs` | Comparações opcionais com saídas do C#, ativadas por `SL_FIXTURES_CSHARP` e `SL_SOLVER_CSHARP` |
| `native/ui/tests` | O modelo da tela em todos os estados, os textos iguais aos da versão HTML, o layout em várias escalas de DPI e o custo do Modo pensar com números enormes |
| `native/app/tests` | A janela real do Windows: resposta ao vivo, esmaecimento, pausa, Enter e setas, AltGr, links, dicas, desenho dos campos e do rodapé, alto contraste, DPI, instalação e preferências |
| `native/linux/tests` | O app Linux com o egui_kittest pela árvore de acessibilidade, a linha de comando e a instalação |
| `tests/PortableTests.cjs` | As 76.674 respostas do JavaScript iguais às do Rust, o texto do Modo pensar igual ao do Rust nos três métodos, o vetor de potências, o corpus diferencial, o modelo da interface, IDs únicos e o HTML completo |
| `tests/InterfaceHtml.cjs` | A versão HTML no Google Chrome sem janela: resposta ao vivo, esmaecimento e anúncios, teclado, erros por campo, IME, 2×2 e 3×3, links do rodapé, Modo pensar, valores longos, movimento reduzido, alto contraste e celular |

### Como a paridade é verificada
1. O `cargo test --release` roda com `SL_FIXTURES_SAIDA=tests\fixtures.tsv`, `SL_PASSOS_SAIDA=tests\passos.tsv` e `SL_POTENCIAS_SAIDA=tests\potencias-corpus.tsv`. O teste de robustez grava a resposta de cada um dos 76.674 sistemas, o do Modo pensar grava o texto dos passos de 609 sistemas nos três métodos, e o das potências grava a leitura de 200.000 entradas aleatórias.
2. O `csc.exe` compila a auditoria do C# legado, que grava `tests\fixtures-csharp.tsv` com os mesmos sistemas. O `fc /b` compara os dois arquivos byte a byte.
3. O `node tools/build-portable.cjs` gera o HTML, e o `node tests/PortableTests.cjs` com os três arquivos confere que o JavaScript dá as mesmas respostas, o mesmo passo a passo e a mesma leitura de cada entrada.
4. O `node tests/InterfaceHtml.cjs` abre o HTML no Chrome sem janela e usa a interface como uma pessoa usaria.
5. Os arquivos temporários são apagados, e o script sai com 0 quando tudo passa ou 1 quando algo falha.

Cada linha das respostas tem o formato `n|entradas|tipo|resposta em base64`, em UTF-8. O Rust e o C# geram arquivos idênticos, com 8.052.533 bytes e SHA-256 `1cc866ab0133fee62657221dac933f5a00601b69fb40d3206018272832673fea`, e o JavaScript confere cada linha desse arquivo.

## Integração contínua
| Workflow | Quando roda | O que faz |
| --- | --- | --- |
| Build | Push e pull request na `main` | No Windows, roda o `testar-tudo.cmd`, compila com `cargo build --release --locked`, calcula o SHA-256 e guarda o exe, o HTML e o `SHA256SUMS.txt` como artefato. No Ubuntu, roda os testes do núcleo, do modelo e do app Linux e guarda o binário Linux com o SHA-256 |
| Lint | Push e pull request na `main` | Confere o rustfmt, o Clippy sem avisos no Windows e no Linux e se o `SistemasLineares.html` commitado é igual ao gerado |
| Release | Tag `vX.Y.Z` | Confere a versão e o CHANGELOG, roda a validação, compila e publica a release com o exe, o HTML, o `SHA256SUMS.txt` e as notas do CHANGELOG; depois testa, compila e anexa o binário Linux e o `SHA256SUMS-linux.txt` |

A toolchain dos workflows vem do `rust-toolchain.toml`, e os scripts de apoio ficam em `.github/scripts`.

## Decisões
- **Frações exatas em vez de ponto flutuante:** em um sistema quase singular, com coeficientes como `1e-50`, o ponto flutuante confunde valores pequenos com zero e classifica errado. Com frações, a classificação é sempre exata.
- **Win32 e Direct2D em vez de WebView ou .NET:** um único exe, sem runtime para instalar, com controles nativos e desenho nítido em qualquer DPI.
- **Um modelo da tela para os dois apps nativos:** o `native/ui` decide o comportamento e os textos, e cada app só desenha. Assim o Windows e o Linux não divergem, e o HTML tem uma referência única para portar.
- **Resposta ao vivo com orçamento:** resolver é instantâneo para os sistemas do dia a dia, mas números de milhares de algarismos podem levar mais tempo. Acima de 8.000 bits, a conta espera o Enter, para a digitação nunca travar.
- **Limite de 300 algarismos nas potências:** potências de números como 101/137 geram frações densas, em que o mdc fica caro. Com o limite, mesmo o pior caso resolve na hora no Rust e no navegador, e os números grandes de sempre (`1e1000`) continuam aceitos.
- **C# mantido como legado:** a versão anterior virou o oráculo de paridade, e qualquer divergência do Rust aparece no `testar-tudo.cmd`.
- **HTML gerado e versionado:** quem não pode rodar o exe abre o HTML direto do repositório, e o Lint garante que ele está em dia.
