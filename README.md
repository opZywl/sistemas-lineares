# Sistemas Lineares — Lucas Lima

Aplicativo para resolver sistemas lineares 2×2 e 3×3. Interface escura inspirada em https://lucas-lima.xyz/, com detalhes lilás, ícones SVG e Space Grotesk incorporada. Não é necessário instalar a fonte.

## Executar

- **SistemasLineares.exe**: aplicativo nativo para Windows, com .NET Framework 4 ou posterior compatível instalado. Compilado em AnyCPU; testado neste computador em processos de 32 e 64 bits. Não requer Python, Node.js ou internet para calcular.
- **SistemasLineares.html**: alternativa portátil. Abra com dois cliques em um navegador atual com suporte a BigInt, como Chrome, Edge, Firefox ou Safari. O HTML inclui o cálculo, os estilos, os SVGs e a fonte no próprio arquivo; funciona offline. Pode ser utilizado em Windows, macOS e Linux com navegador compatível.

Não existe garantia universal de execução em qualquer computador: sistema operacional, runtime, navegador e políticas de execução precisam ser compatíveis. A versão HTML oferece uma alternativa quando o executável Windows não puder ser utilizado. A validação de 32 e 64 bits foi realizada no mesmo Windows, não em todos os modelos de computador ou sistemas operacionais.

## Usar

1. Escolha **2 × 2** ou **3 × 3**.
2. Preencha o coeficiente de cada variável e o termo independente, à direita do sinal de igualdade. Use **0** para variáveis que não aparecem na equação.
3. Clique em **Resolver sistema** ou pressione **Enter**. **Tab** navega entre os campos.
4. Confira o painel de resultado. **Copiar resultado** copia a resposta completa, inclusive valores longos.

**Usar exemplo** preenche um sistema para experimentar. **Limpar** apaga os campos. **Feito por Lucas Lima**, no topo, abre o site do autor quando clicado. O programa não faz consultas externas para resolver o sistema e não utiliza modelos de IA.

### Formatos de entrada

Aceita inteiros (`-2`), decimais (`1,5` ou `1.5`), frações (`3/4`) e notação científica (`1e-6`). As frações também podem ter decimais, como `2,5/0,5`. Cada campo deve ter até 100 caracteres; expoentes devem estar entre -1000 e 1000. Denominador zero, campos vazios e formatos inválidos são sinalizados. Não há separadores de milhar; utilize números sem agrupamento. Expressões simbólicas como `√2` não são aceitas.

O algoritmo trabalha com coeficientes racionais e não usa aproximação de ponto flutuante. A solução aparece como inteiro ou fração exata. O programa distingue solução única, infinitas soluções e sistemas sem solução. Nas soluções gerais, `t1`, `t2`, etc. são parâmetros reais livres.

## Código e compilação

- **src/Solver.cs**: racional de precisão arbitrária e eliminação de Gauss-Jordan.
- **src/Program.cs**: interface Windows Forms, validação e ações.
- **src/Design.cs**: componentes e renderização dos SVGs.
- **src/Fonts.cs**: carregamento local das fontes incorporadas.
- **assets/**: fontes, licença, SVGs, ícone e manifesto.
- **portable/**: fontes da versão HTML, incluindo o cálculo em JavaScript/BigInt.
- **tools/build-portable.cjs**: gera o HTML independente.
- **tests/**: testes matemáticos, de interface e de paridade entre versões.

Feche o aplicativo e abra **compilar.cmd** para compilar e testar a versão Windows com o compilador do .NET Framework. Execute **testar-tudo.cmd** para incluir a auditoria independente e, quando houver Node.js instalado, reconstruir e testar a versão portátil. Os runtimes de desenvolvimento não são necessários para usar os arquivos já compilados.

## Validação

- 3.421 verificações matemáticas básicas.
- 76.674 sistemas auditados com posto calculado por menores e soluções únicas comparadas com a regra de Cramer: 1.703.886 verificações independentes.
- O mesmo conjunto de 76.674 sistemas comparado entre C# e JavaScript, incluindo a resposta completa e a parametrização das infinitas soluções.
- Testes de interface em processos de 32 e 64 bits, além de conferência visual e de um exemplo 3×3 nas duas interfaces.

O conjunto auditado inclui todos os sistemas 2×2 com entradas inteiras entre -2 e 2; todas as matrizes de coeficientes 3×3 com entradas -1, 0 e 1, combinadas com três vetores de termos independentes; e 2.000 sistemas aleatórios adicionais. Há testes específicos para números de 100 dígitos, casos quase singulares, formatos inválidos e expoentes. Esses testes constituem evidência verificável, não uma promessa de ausência de falhas em todas as entradas ou ambientes possíveis.

Consulte **APRESENTACAO.md** para entender o método e demonstrar exemplos ao professor. A Space Grotesk é distribuída com sua licença em assets/SpaceGrotesk-LICENSE.txt; fonte original: https://github.com/floriankarsten/space-grotesk.
