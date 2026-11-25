# Como apresentar o projeto

Lucas Lima · RA 185964

O objetivo é receber os coeficientes de um sistema linear 2×2 ou 3×3 e apresentar sua classificação e solução. A interface representa cada equação na forma a·x + b·y = c, ou a·x + b·y + c·z = d.

## Método matemático

O programa monta a matriz aumentada [A | b] e aplica eliminação de Gauss-Jordan. Procura um coeficiente não nulo para servir de pivô; troca linhas quando necessário; divide a linha pelo pivô; e elimina os demais coeficientes da coluna. Repete o processo nas próximas colunas. Essas operações preservam o conjunto de soluções.

Uma linha com todos os coeficientes zero e termo independente diferente de zero representa uma contradição, como 0 = 1: o sistema não tem solução. Quando todas as variáveis têm pivô, a solução é única. Se não houver contradição e restarem variáveis sem pivô, elas se tornam parâmetros livres, e o sistema tem infinitas soluções.

Os números são armazenados como frações reduzidas de inteiros de precisão arbitrária: `BigInt` da crate num-bigint nos apps nativos em Rust e BigInt na versão HTML em JavaScript. Isso evita confundir uma diferença pequena com zero por arredondamento. Não há IA na resolução: o resultado é produzido pelas operações matemáticas descritas no código.

## Entrada e resposta automática

Cada campo aceita inteiros, decimais, frações, notação científica e potências, como `2^3`, `10^-2`, `(1/2)^3` e `10²`. A potência vale antes do sinal, como na matemática: `-2²` é −4 e `(-2)²` é 4. Com o campo focado, a dica ao lado do gráfico mostra o valor lido, por exemplo "Este campo vale 8.".

Não há botão de resolver: a resposta aparece assim que o último campo é preenchido. Enquanto se digita, o card mostra quantos campos faltam e a prévia das equações, com `?` no lugar do que falta. Enter leva ao próximo campo vazio, e as setas para cima e para baixo mudam de equação.

## Gráfico do sistema

Abaixo das equações, o gráfico mostra o sistema enquanto se digita. No 2×2, cada equação a·x + b·y = c é uma reta, e a solução única é o ponto onde as retas se cruzam, com as coordenadas marcadas nos eixos. Retas paralelas nunca se cruzam (sistema sem solução), e retas coincidentes se sobrepõem (infinitas soluções). No 3×3, cada equação é um plano; a solução única é o ponto comum aos três, e as linhas claras mostram onde cada par de planos se encontra.

Cada reta ou plano tem a cor do número da sua equação, e a legenda ao lado diz o que o gráfico mostra. A classificação vem da conta exata; o desenho só ilustra. Em uma apresentação, use o exemplo 2×2 e mude o termo independente de uma equação: a reta desliza e o ponto da solução acompanha.

## Modo pensar

Com a resposta na tela, o botão **Modo pensar**, em destaque no card de resultado, abre o passo a passo. No alto da página, escolha o método:

- **Gauss-Jordan:** zera a coluna de cada pivô em todas as linhas até a parte da esquerda virar a matriz identidade. Cada passo mostra a operação (por exemplo, L₂ ← L₂ − 2·L₁), a conta feita na linha e a matriz resultante.
- **Regra de Cramer:** calcula D = det(A) e, para cada incógnita, o determinante com a coluna dela trocada pelos termos independentes. No 2×2 usa a diagonal principal menos a secundária e no 3×3 a regra de Sarrus. Quando D = 0, o método não decide entre infinitas soluções e nenhuma, e o passo a passo continua por escalonamento.
- **Escalonamento de Gauss:** zera só o que está abaixo de cada pivô, com o multiplicador m = valor ÷ pivô, e resolve de baixo para cima por substituição.

Os três métodos chegam à mesma resposta, terminam conferindo os valores nas equações originais e classificam o sistema em SPD, SPI ou SI. Os links do rodapé, **Gauss-Jordan**, **Cramer** e **Escalonamento**, abrem o Modo pensar direto no método. Em uma apresentação, resolva o exemplo 3×3 e alterne entre os três métodos para comparar os caminhos.

## Demonstrações

| Caso | Equações | Resultado esperado |
|---|---|---|
| Única 2×2 | 2x + y = 5; x − y = 1 | x = 2; y = 1 |
| Única 3×3 | x + y + z = 6; 2x − y + z = 3; x + 2y − z = 2 | x = 1; y = 2; z = 3 |
| Infinitas | x + y = 2; 2x + 2y = 4 | x = 2 − t1; y = t1 |
| Sem solução | x + y = 2; 2x + 2y = 5 | Sistema incompatível |
| Troca de linhas | 0x + y = 2; x + 0y = 3 | x = 3; y = 2 |
| Fração exata | 3x + 0y = 1; 0x + 2y = 1 | x = 1/3; y = 1/2 |
| Todas livres | 0x + 0y = 0; 0x + 0y = 0 | x = t1; y = t2 |
| Com potências | 2³x + y = 9, digitada como `2^3`, `1` e `9`; x − y = 0 | x = 1; y = 1 |

Mostre também como o app ajuda a corrigir a entrada:

- Com campos vazios, o card mostra "Faltam N campos" e a prévia das equações. Enter leva ao próximo campo vazio.
- Digite `1/0` num campo e saia dele: o campo fica vermelho e a dica diz "O denominador não pode ser zero.". O mesmo aviso aparece se você pressionar Enter no campo ou parar de digitar por um instante.
- Digite `2^3` num campo: a dica mostra "Este campo vale 8.".

## Onde explicar no código

Em native/core/src/lib.rs, `Rational::parse` interpreta os números, e native/core/src/power.rs lê as potências e os parênteses; a estrutura `Rational` implementa as operações exatas; `solve` efetua a eliminação; e `format` escreve as respostas, inclusive com parâmetros livres. Em native/core/src/steps.rs, `walkthrough` monta os passos do Modo pensar para cada método; native/core/tests/passos.rs refaz cada operação de linha, recalcula cada determinante pela definição e confere que os três métodos chegam à resposta do `solve`. Em native/core/tests/robustez.rs, os resultados são conferidos por métodos independentes: posto por menores, regra de Cramer e substituição nas equações originais. O estado da tela, com a resposta automática e os avisos, fica em native/ui/src/model.rs e é o mesmo nos apps de Windows e de Linux. O gráfico é calculado em native/ui/src/graph.rs (retas, planos recortados pela caixa, legenda e animação) e desenhado do mesmo jeito no Windows, no Linux e no HTML, cujo porte fica em portable/graph.js. A janela do Windows fica em native/app/src/window.rs, e a do Linux em native/linux/src/app.rs.

A versão anterior, em C#, fica em legado/csharp como implementação de referência: as duas dão as mesmas respostas, byte a byte.

Antes da apresentação, abra a versão escolhida no computador que será utilizado e execute os exemplos acima. Leve o SistemasLineares.exe (Windows) ou o sistemas-lineares (Linux), baixados da página de Releases, e o SistemasLineares.html offline. Os apps são arquivos únicos e não precisam de instalação. Consulte o README para os requisitos de execução e formatos de entrada.
