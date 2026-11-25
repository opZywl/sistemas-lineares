# Changelog

Todas as mudanças relevantes do Sistemas Lineares ficam registradas neste arquivo.

O formato segue o [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o projeto adota o [Versionamento Semântico](https://semver.org/lang/pt-BR/spec/v2.0.0.html).

## [Não lançado]

### Adicionado
- **Modo pensar**
  - Depois de resolver, o botão Modo pensar mostra como a resposta foi calculada, passo a passo, com o porquê de cada operação.
  - Três métodos à escolha: Gauss-Jordan, regra de Cramer e escalonamento de Gauss com substituição de baixo para cima.
  - Cada passo mostra a operação de linha, a conta feita em cada linha e a matriz resultante, com o pivô contornado e as linhas alteradas em destaque.
  - Na regra de Cramer, os determinantes saem pela diagonal no 2×2 e pela regra de Sarrus no 3×3. Com D = 0, o app explica por que o método não decide o sistema e continua por escalonamento.
  - O passo a passo termina conferindo a resposta nas equações originais. Com infinitas soluções, confere uma solução particular.
  - Classificação do sistema em SPD, SPI ou SI.
  - Botão Copiar passos, que copia o passo a passo inteiro como texto.
  - O último método escolhido fica salvo entre aberturas.
  - Leitores de tela leem cada passo, e o título do método é anunciado quando muda.
- **Núcleo:** `walkthrough`, que gera os passos de cada método com aritmética exata, e testes que refazem cada operação de linha e recalculam cada determinante de forma independente em mais de 50 mil sistemas. A conversão `Rational::to_f64`, usada só para desenhar o gráfico, dá o mesmo valor no Rust e no JavaScript.
- **Versão HTML:** o mesmo Modo pensar, com o texto dos passos idêntico ao do app nativo.
- **Potências na entrada**
  - `2^3`, `2**3`, `10^-2`, `(1/2)^3`, `(-2)^2` e expoentes sobrescritos como `10²` e `2⁻¹`. No teclado ABNT2, AltGr+2 e AltGr+3 digitam ² e ³.
  - A potência vale antes do sinal, como na matemática: `-2²` é −4 e `(-2)²` é 4. Potência de potência pede parênteses, como `(2^3)^2`.
  - Expoente inteiro de -1000 a 1000, e cada valor com potência pode ter até 300 algarismos.
  - Mensagens próprias para 0⁰, 0 elevado a expoente negativo, expoente inválido, potência de potência sem parênteses e multiplicação, como em `1,5×10³`.
  - Com o campo focado, a dica mostra o valor lido, como "Este campo vale -4.".
- **Resposta automática**
  - A solução aparece assim que todos os campos estão preenchidos, sem Enter e sem botão de resolver.
  - Enquanto você digita, a resposta anterior fica esmaecida, com o aviso "Atualizando", e o card se ajusta numa pausa da digitação ou ao sair do campo.
  - Com números muito grandes, a conta espera o Enter em um dos campos, para a digitação nunca travar.
  - O card mostra o progresso, como "Faltam 3 campos" ou "Confira os valores", com a prévia das equações e `?` no lugar do que falta.
- **Teclado**
  - Enter leva ao próximo campo vazio e resolve quando tudo está preenchido.
  - As setas para cima e para baixo mudam de equação na mesma coluna.
- **Gráfico do sistema**
  - O espaço entre as equações e os botões mostra o sistema desenhado, ao vivo: no 2×2, cada equação é uma reta no plano cartesiano; no 3×3, um plano dentro de uma caixa 3D.
  - Cada reta ou plano tem a cor do número da sua equação: lilás, menta e pêssego.
  - A solução única aparece como um ponto que surge com um pulso, com linhas-guia até os eixos e as coordenadas no 2×2 e uma guia até o chão da caixa no 3×3. No 3×3, linhas claras mostram onde cada par de planos se encontra.
  - Infinitas soluções também aparecem: pontos percorrem a reta solução, o plano solução ganha contorno, e quando qualquer ponto serve a textura inteira acende.
  - Ao digitar, as retas e os planos surgem, deslizam e somem com animação, e a área do gráfico se ajusta sozinha para mostrar a origem, a solução e cada equação.
  - Uma legenda explica o que o gráfico mostra, como "As retas se cruzam em (2, 1): esse ponto é a solução do sistema.", inclusive com retas paralelas ou coincidentes e equações do tipo 0 = 0 ou 0 = 5.
  - As coordenadas ficam sempre legíveis: os rótulos fogem das retas, das letras dos eixos e do próprio ponto, e a grade fica nítida em qualquer escala de tela.
  - A legenda não quebra a linha no meio de pares como (2, 1) nem de igualdades como 0 = 5/2.
  - Respeita a preferência de menos movimento e o alto contraste do sistema, e os leitores de tela leem o gráfico como uma imagem descrita pela legenda.
- **Rodapé interativo:** "Passo a passo por Gauss-Jordan · Cramer · Escalonamento" abre o Modo pensar direto no método. Ao focar ou passar o mouse num método, o rodapé explica como ele funciona. As teclas Enter, ↑ ↓ e Esc aparecem desenhadas.
- **Dicas:** Modo pensar, Copiar, Limpar e Usar exemplo explicam o que fazem, inclusive quando estão desabilitados.
- **App para Linux** (`native/linux`, com eframe e egui)
  - O mesmo visual e o mesmo comportamento do app para Windows.
  - Leitor de tela pelo AT-SPI e alto contraste.
  - Instalação por usuário com `--instalar`, com atalho no menu de aplicativos.
  - `SISTEMAS_LINEARES_ANIMACOES=0` desliga as animações do gráfico, que também seguem a preferência do desktop.
  - `abrir.sh` compila, testa e abre o app.
- **Testes**
  - Vetor de potências compartilhado entre Rust e JavaScript, conferido também por um gerador independente.
  - Corpus diferencial com 200 mil entradas.
  - Interface da versão HTML testada no Google Chrome sem janela.
  - Testes de interface do app Linux com o egui_kittest.
  - Corpus de 1.844 cenas do gráfico, com a mesma legenda e o mesmo desenho, byte a byte, no Rust e no JavaScript.
  - Propriedades do gráfico: nada sai da área desenhada, e 300 animações sorteadas, também em alto contraste, terminam exatamente na cena de repouso.

### Alterado
- O botão Copiar resultado virou Copiar e divide o rodapé do card de resultado com o Modo pensar.
- Interface mais limpa
  - Os seletores 2×2/3×3 e de método ficam num trilho, com o item escolhido em lilás.
  - O anel de foco do teclado tem 2 px.
  - O campo focado ganha um halo, e o campo sob o mouse destaca a borda.
  - O número da equação ganha a cor da sua reta ou do seu plano no gráfico quando a linha está completa.
- Cada tamanho guarda o próprio sistema: trocar entre 2×2 e 3×3 não mistura valores, e Limpar apaga os dois.
- O erro de um campo aparece ao sair dele, no Enter ou numa pausa da digitação, e não pisca enquanto o campo é corrigido.
- Usar exemplo mostra a resposta na hora e não tira o foco do lugar.
- Com a resposta na tela, o Modo pensar fica em destaque no card, e o Copiar vira um botão secundário.
- A dica embaixo dos campos virou a legenda do gráfico. Erros, avisos e o valor lido de uma potência continuam aparecendo no mesmo lugar, e o aviso de exemplo resolvido deu lugar à descrição da solução no gráfico.
- O workspace Rust ganhou o crate `native/ui`, com o modelo da tela, o layout, o tema e o Modo pensar compartilhados pelos apps de Windows e de Linux.

### Corrigido
- O botão Limpar ficou mais largo, e o ícone e o texto não encostam mais nas bordas.
- Ctrl+A e Ctrl+Backspace não disparam mais com AltGr. No teclado US-Internacional, AltGr+A selecionava e apagava o campo.
- O Modo pensar não trava mais a janela com números de mil algarismos.
- Versão HTML: a leitura dos números ficou idêntica à do app nativo, inclusive nos espaços Unicode e nas mensagens de expoente.

### Removido
- O botão Resolver sistema: a resposta aparece sozinha, e o Enter num campo resolve os sistemas com números muito grandes.
- O texto fixo de ajuda embaixo dos campos, com os formatos aceitos. Os formatos continuam na descrição de cada campo para leitores de tela, e o lugar dele ficou com o gráfico e a legenda.
- O `legado/csharp/README.md`.

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
