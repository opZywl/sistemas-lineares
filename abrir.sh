#!/usr/bin/env bash
set -u

RAIZ="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_TOML="$RAIZ/native/Cargo.toml"
HTML="$RAIZ/SistemasLineares.html"
MODO="app"
VERDE=$'\033[92m'
VERMELHO=$'\033[91m'
AMARELO=$'\033[93m'
NORMAL=$'\033[0m'

for argumento in "$@"; do
    case "${argumento,,}" in
        html) MODO="html" ;;
        instalar) MODO="instalar" ;;
        desinstalar) MODO="desinstalar" ;;
        -h|--ajuda|--help) MODO="ajuda" ;;
        *) echo "${AMARELO} AVISO ${NORMAL} Opção desconhecida ignorada: $argumento" ;;
    esac
done

falha() {
    echo
    echo "${VERMELHO} ERRO ${NORMAL} $1"
    exit 1
}

aviso() {
    echo "${AMARELO} AVISO ${NORMAL} $1"
}

ok() {
    echo
    echo "${VERDE} OK ${NORMAL} $1"
}

if [ "$MODO" = "ajuda" ]; then
    cat <<'TEXTO'
Uso: ./abrir.sh [html | instalar | desinstalar]

  (sem opção)   Confere o Rust, compila, testa e abre o Sistemas Lineares para Linux.
  html          Atualiza e abre a versão HTML offline no Google Chrome.
  instalar      Compila, testa e instala para o seu usuário (menu de aplicativos e ~/.local/bin).
  desinstalar   Remove a instalação do seu usuário.
TEXTO
    exit 0
fi

echo
echo " Sistemas Lineares - Lucas Lima - RA 185964"
echo

chrome() {
    for candidato in google-chrome google-chrome-stable chromium chromium-browser; do
        if command -v "$candidato" >/dev/null 2>&1; then
            command -v "$candidato"
            return 0
        fi
    done
    return 1
}

if [ "$MODO" = "html" ]; then
    echo "[1/2] Atualizando a versão HTML..."
    if command -v node >/dev/null 2>&1; then
        node "$RAIZ/tools/build-portable.cjs" || falha "Não foi possível gerar o SistemasLineares.html."
    else
        aviso "Node.js não encontrado: usando o SistemasLineares.html já gerado."
    fi
    [ -f "$HTML" ] || falha "SistemasLineares.html não encontrado."
    echo "[2/2] Abrindo no Chrome..."
    navegador="$(chrome)" || falha "Google Chrome não encontrado. Abra o SistemasLineares.html em um navegador atual."
    setsid "$navegador" "$HTML" >/dev/null 2>&1 &
    ok "Versão HTML aberta no Chrome: $HTML"
    exit 0
fi

if [ -z "${CARGO_TARGET_DIR:-}" ]; then
    case "$RAIZ" in
        /mnt/*) export CARGO_TARGET_DIR="$HOME/.cache/sistemas-lineares/target" ;;
        *) export CARGO_TARGET_DIR="$RAIZ/native/target" ;;
    esac
fi
BINARIO="$CARGO_TARGET_DIR/release/sistemas-lineares"

if [ "$MODO" = "desinstalar" ]; then
    INSTALADO="$HOME/.local/bin/sistemas-lineares"
    alvo=""
    if [ -x "$INSTALADO" ]; then
        alvo="$INSTALADO"
    elif [ -x "$BINARIO" ]; then
        alvo="$BINARIO"
    fi
    [ -n "$alvo" ] || falha "Nenhuma instalação do Sistemas Lineares encontrada para remover."
    echo "[1/1] Removendo a instalação do seu usuário..."
    "$alvo" --desinstalar
    codigo=$?
    if [ "$codigo" -eq 2 ]; then
        echo
        aviso "Desinstalação cancelada."
        exit 0
    fi
    [ "$codigo" -eq 0 ] || falha "A desinstalação não foi concluída (veja a mensagem acima)."
    ok "Sistemas Lineares removido do menu de aplicativos e de ~/.local/bin."
    exit 0
fi

echo "[1/5] Verificando o Rust e o compilador C..."
command -v cargo >/dev/null 2>&1 || falha "Rust não encontrado. Instale com: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
command -v cc >/dev/null 2>&1 || falha "Compilador C não encontrado. No Ubuntu ou Debian: sudo apt install build-essential pkg-config"
if [ "$MODO" = "app" ] && [ -z "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ]; then
    falha "Nenhuma tela gráfica encontrada (DISPLAY e WAYLAND_DISPLAY vazios). Rode em uma sessão gráfica ou use: ./abrir.sh html"
fi

echo "[2/5] Compilando a versão mais recente (na primeira vez demora alguns minutos)..."
cargo build --release --manifest-path "$CARGO_TOML" -p sistemas-lineares-linux \
    || falha "A compilação falhou (veja acima). No Ubuntu ou Debian: sudo apt install build-essential pkg-config"

echo "[3/5] Rodando os testes..."
cargo test --release --quiet --manifest-path "$CARGO_TOML" \
    -p sistemas-lineares-core -p sistemas-lineares-ui -p sistemas-lineares-linux \
    || falha "Os testes falharam (veja acima). Nada foi instalado nem aberto."

[ -x "$BINARIO" ] || falha "Binário não encontrado em $BINARIO."
echo "[4/5] Pronto: $BINARIO"

if [ "$MODO" = "instalar" ]; then
    echo "[5/5] Instalando para o seu usuário (menu de aplicativos e ~/.local/bin)..."
    "$BINARIO" --instalar || falha "A instalação não foi concluída (veja a mensagem acima)."
    ok "Instalado em ~/.local/bin/sistemas-lineares."
    echo "      Abra pelo menu de aplicativos: Sistemas Lineares."
    echo "      Para remover: ./abrir.sh desinstalar"
    exit 0
fi

echo "[5/5] Abrindo o aplicativo..."
setsid "$BINARIO" >/dev/null 2>&1 &
processo=$!
for _ in 1 2 3 4 5 6; do
    sleep 0.5
    kill -0 "$processo" 2>/dev/null || falha "O aplicativo fechou logo ao abrir. Rode $BINARIO no terminal para ver a mensagem."
done
ok "Sistemas Lineares aberto (processo $processo): $BINARIO"
echo "      Para fechar, feche a janela do aplicativo."
echo "      Versão HTML offline no Chrome: ./abrir.sh html"
