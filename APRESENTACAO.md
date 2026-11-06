# Como apresentar o projeto

O objetivo é receber os coeficientes de um sistema linear 2×2 ou 3×3 e apresentar sua classificação e solução. A interface representa cada equação na forma a·x + b·y = c, ou a·x + b·y + c·z = d.

## Método matemático

O programa monta a matriz aumentada [A | b] e aplica eliminação de Gauss-Jordan. Procura um coeficiente não nulo para servir de pivô; troca linhas quando necessário; divide a linha pelo pivô; e elimina os demais coeficientes da coluna. Repete o processo nas próximas colunas. Essas operações preservam o conjunto de soluções.

Uma linha com todos os coeficientes zero e termo independente diferente de zero representa uma contradição, como 0 = 1: o sistema não tem solução. Quando todas as variáveis têm pivô, a solução é única. Se não houver contradição e restarem variáveis sem pivô, elas se tornam parâmetros livres, e o sistema tem infinitas soluções.

Os números são armazenados como frações reduzidas de inteiros de precisão arbitrária: BigInteger em C# e BigInt em JavaScript. Isso evita confundir uma diferença pequena com zero por arredondamento. Não há IA na resolução: o resultado é produzido pelas operações matemáticas descritas no código.

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

Mostre também o aviso para um campo vazio ou para uma fração com denominador zero. O programa solicita a correção do campo antes de calcular.

## Onde explicar no código

Em src/Solver.cs, Rational.Parse interpreta os números; a estrutura Rational implementa as operações exatas; Solver.Solve efetua a eliminação; e Solver.Format escreve as respostas, inclusive com parâmetros livres. Em tests/RobustnessTests.cs, os resultados são conferidos por métodos independentes: determinantes e substituição nas equações originais.

Antes da apresentação, abra a versão escolhida no computador que será utilizado e execute os exemplos acima. Leve tanto o EXE quanto o HTML offline. Consulte o README para os requisitos de execução e formatos de entrada.
