# Sistemas Lineares
![Plataformas: Windows e Linux](https://img.shields.io/badge/plataforma-Windows%20%7C%20Linux-CEA9F3?style=flat-square&labelColor=111010)\
Resolve sistemas lineares 2×2 e 3×3 com cálculo exato, em apps nativos para Windows e Linux.

Site: [lucas-lima.xyz](https://lucas-lima.xyz)\
Última versão: [Releases](https://github.com/opZywl/sistemas-lineares/releases/latest)\
Autor: Lucas Lima · RA 185964

## Como instalar no Windows?
O app roda no Windows 10 e 11 de 64 bits. Não precisa de .NET, de navegador nem de internet.

- **Passo 1:** Baixe o `SistemasLineares-Windows.exe` da [última release](https://github.com/opZywl/sistemas-lineares/releases/latest).
- **Passo 2:** Abra o arquivo com dois cliques. Ele é portátil e roda da pasta onde estiver.
- **Passo 3 (opcional):** Instale para ter o atalho no menu Iniciar, como explicado abaixo.

### Aviso do SmartScreen
O executável não tem assinatura digital. Por isso o navegador pode avisar que o arquivo não é baixado com frequência, e o Windows pode mostrar **O Windows protegeu o computador** na primeira execução. Nesse caso, clique em **Mais informações** e depois em **Executar assim mesmo**. Se quiser, confira antes o SHA-256 do arquivo, como explicado abaixo.

### Instalar e desinstalar no Windows
A instalação é por usuário e não pede administrador. Ela copia o app para `%LOCALAPPDATA%\Programs\Sistemas Lineares`, cria o atalho **Sistemas Lineares** no menu Iniciar e registra o app em **Configurações > Aplicativos > Aplicativos instalados**.

- **Instalar:** no PowerShell, na pasta do download, rode `.\SistemasLineares-Windows.exe --instalar`. Com o repositório clonado, rode `abrir.cmd instalar`.
- **Desinstalar:** em **Aplicativos instalados**, escolha **Sistemas Lineares** e depois **Desinstalar**. `.\SistemasLineares-Windows.exe --desinstalar` e `abrir.cmd desinstalar` fazem o mesmo.

Instalar de novo por cima atualiza a cópia instalada. A desinstalação remove a pasta, o atalho, a entrada em Aplicativos instalados e as preferências salvas.

## Como instalar no Linux?
O app roda em distribuições Linux de 64 bits com X11 ou Wayland, como Ubuntu, Debian, Fedora e Arch.

- **Passo 1:** Baixe o `SistemasLineares-Linux` da [última release](https://github.com/opZywl/sistemas-lineares/releases/latest).
- **Passo 2:** No terminal, na pasta do download, torne o arquivo executável e abra o app:

```bash
chmod +x SistemasLineares-Linux
./SistemasLineares-Linux
```

- **Passo 3 (opcional):** Instale para ter o atalho no menu de aplicativos com `./SistemasLineares-Linux --instalar`.

A instalação é por usuário e não pede `sudo`. Ela copia o app para `~/.local/bin/sistemas-lineares` e cria o atalho e os ícones nas pastas do usuário; depois disso, o comando é `sistemas-lineares`. Para remover, rode `sistemas-lineares --desinstalar`, que também apaga as preferências salvas. Com o repositório clonado, `./abrir.sh instalar` e `./abrir.sh desinstalar` fazem o mesmo.

Para abrir com as cores de alto contraste, use `sistemas-lineares --alto-contraste`. O app também segue o alto contraste do sistema quando o desktop informa essa preferência, como no GNOME.

## Usando
1. Escolha **2 × 2** ou **3 × 3**. Cada tamanho guarda o próprio sistema, então trocar de um para o outro não mistura os valores.
2. Preencha o coeficiente de cada variável e o termo independente, à direita do sinal de igual. Use **0** para a variável que não aparece na equação.
3. A resposta aparece sozinha assim que o último campo é preenchido. Enquanto você digita, o card mostra quantos campos faltam e a prévia das equações.
4. Clique em **Copiar** para copiar a resposta completa, inclusive valores longos, ou em **Modo pensar** para ver como ela foi calculada.

**Usar exemplo** preenche um sistema pronto, e **Limpar** apaga os campos dos dois tamanhos.

### Teclado
| Tecla | O que faz |
| --- | --- |
| Enter | Vai ao próximo campo vazio. Com tudo preenchido, resolve. |
| ↑ e ↓ | Mudam de equação, na mesma coluna. |
| Tab e Shift+Tab | Passam pelos campos e botões. |
| Esc | No Modo pensar, volta para o sistema. |

No rodapé, **Gauss-Jordan**, **Cramer** e **Escalonamento** abrem o Modo pensar direto no método. Ao focar um deles ou passar o mouse, o rodapé explica como o método funciona.

### Formatos aceitos
- Inteiros (`-2`), decimais com vírgula ou ponto (`1,5` ou `1.5`), frações (`3/4`, inclusive `2,5/0,5`) e notação científica (`1e-6`).
- Potências com `^` ou `**` (`2^3`, `10^-2`, `(1/2)^3`) e expoentes sobrescritos (`10²`, `2⁻¹`). No teclado ABNT2, AltGr+2 e AltGr+3 digitam ² e ³.
- A potência vale antes do sinal: `-2²` é −4 e `(-2)²` é 4. Potência de potência pede parênteses, como `(2^3)^2`.
- Com potência ou parênteses no campo, a dica abaixo dos campos mostra o valor lido, como "Este campo vale 8.".
- Cada campo aceita até 100 caracteres. O expoente da notação científica e o da potência vão de -1000 a 1000, e cada valor com potência pode ter até 300 algarismos.
- Não use separador de milhar nem contas: para 1,5×10³, digite `1,5e3` ou `1500`. Expressões como `√2` não são aceitas.

Campo vazio, denominador zero e formato inválido são apontados no próprio campo quando você sai dele, pressiona Enter ou para de digitar por um instante.

## Versão HTML
O `SistemasLineares-Navegador.html` das releases, que é o mesmo `SistemasLineares.html` da raiz do repositório, faz o mesmo cálculo no navegador, com a mesma resposta automática e o mesmo Modo pensar. Abra com dois cliques em um navegador atual (Chrome, Edge, Firefox ou Safari) no Windows, macOS ou Linux. Ele funciona offline, porque o cálculo, os estilos, os ícones e a fonte estão dentro do arquivo. Com o repositório clonado, `abrir.cmd html` ou `./abrir.sh html` abrem a versão HTML no Chrome.

## Conferindo o SHA-256
Cada release traz o `SHA256SUMS.txt`, com o SHA-256 dos três arquivos, e a mesma tabela nas notas. No PowerShell:

```powershell
Get-FileHash .\SistemasLineares-Windows.exe -Algorithm SHA256
```

No Prompt de Comando, use `certutil -hashfile SistemasLineares-Windows.exe SHA256`. O valor tem que ser igual ao da release; maiúsculas e minúsculas não fazem diferença.

Com o `SHA256SUMS.txt` na mesma pasta, este comando do PowerShell confere todos os arquivos baixados e mostra `OK` ou `DIFERENTE` para cada um:

```powershell
Get-Content .\SHA256SUMS.txt | ForEach-Object { $hash, $nome = $_ -split '\s+', 2; if (Test-Path $nome) { "${nome}: " + $(if ((Get-FileHash $nome -Algorithm SHA256).Hash -eq $hash) { 'OK' } else { 'DIFERENTE' }) } }
```

No Linux, com o `SHA256SUMS.txt` na mesma pasta, rode `sha256sum -c --ignore-missing SHA256SUMS.txt`.

## Onde o Sistemas Lineares guarda dados
- **Windows:** a posição e o tamanho da janela, se ela estava maximizada, a dimensão 2×2 ou 3×3 e o último método do Modo pensar ficam em `HKCU\Software\Lucas Lima\Sistemas Lineares`. A cópia instalada fica em `%LOCALAPPDATA%\Programs\Sistemas Lineares`.
- **Linux:** o tamanho e a posição da janela, a dimensão 2×2 ou 3×3 e o último método ficam em `~/.config/sistemas-lineares/preferencias.ron`. A cópia instalada fica em `~/.local/bin/sistemas-lineares`.

Os valores digitados não são guardados, e nada é enviado pela internet.
