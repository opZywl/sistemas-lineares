use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use sistemas_lineares_linux::install::{self, ICONS, Targets};

struct Saida {
    codigo: i32,
    texto: String,
    erros: String,
}

fn pasta(nome: &str) -> PathBuf {
    let pasta = std::env::temp_dir().join(format!(
        "sl-instalador-{nome}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |tempo| tempo.as_nanos())
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    pasta
}

fn rodar(home: &Path, xdg: Option<(&Path, &Path)>, argumentos: &[&str], entrada: &str) -> Saida {
    let mut comando = Command::new(env!("CARGO_BIN_EXE_sistemas-lineares"));
    comando
        .args(argumentos)
        .env("HOME", home)
        .env_remove("XDG_DATA_HOME")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("SISTEMAS_LINEARES_ALTO_CONTRASTE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some((dados, config)) = xdg {
        comando
            .env("XDG_DATA_HOME", dados)
            .env("XDG_CONFIG_HOME", config);
    }
    let mut filho = comando.spawn().expect("binário");
    if let Some(mut stdin) = filho.stdin.take() {
        let _ = stdin.write_all(entrada.as_bytes());
    }
    let saida = filho.wait_with_output().expect("saída");
    Saida {
        codigo: saida.status.code().unwrap_or(-1),
        texto: String::from_utf8_lossy(&saida.stdout).into_owned(),
        erros: String::from_utf8_lossy(&saida.stderr).into_owned(),
    }
}

fn alvos(home: &Path) -> Targets {
    let home = home.to_path_buf();
    Targets::from_env(move |chave| (chave == "HOME").then(|| home.clone().into_os_string()))
        .unwrap()
}

#[test]
fn versao_e_ajuda() {
    let home = pasta("versao");
    let versao = rodar(&home, None, &["--versao"], "");
    assert_eq!(versao.codigo, 0);
    assert_eq!(
        versao.texto.trim(),
        format!("Sistemas Lineares {}", env!("CARGO_PKG_VERSION"))
    );
    let ajuda = rodar(&home, None, &["--ajuda"], "");
    assert_eq!(ajuda.codigo, 0);
    assert!(ajuda.texto.contains("Uso: sistemas-lineares [opções]"));
    assert!(ajuda.texto.contains("--desinstalar"));
    let estranho = rodar(&home, None, &["--qualquer", "--versao"], "");
    assert_eq!(estranho.codigo, 0);
    assert!(estranho.erros.contains("Opção desconhecida: --qualquer"));
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn instala_e_desinstala_pelo_binario() {
    let home = pasta("ciclo");
    let alvo = alvos(&home);
    let nada = rodar(&home, None, &["--desinstalar"], "");
    assert_eq!(nada.codigo, 0);
    assert!(nada.texto.contains("não está instalado"), "{}", nada.texto);
    let silencioso = rodar(&home, None, &["--instalar", "--silencioso"], "");
    assert_eq!(silencioso.codigo, 0, "{}", silencioso.erros);
    assert!(silencioso.texto.is_empty());
    assert!(alvo.is_installed());
    assert_eq!(
        std::fs::read(alvo.exe()).unwrap(),
        std::fs::read(env!("CARGO_BIN_EXE_sistemas-lineares")).unwrap()
    );
    let atalho = std::fs::read_to_string(alvo.desktop()).unwrap();
    for linha in [
        "Name=Sistemas Lineares",
        "Icon=sistemas-lineares",
        "Terminal=false",
        "Categories=Education;Science;Math;",
        "StartupWMClass=sistemas-lineares",
    ] {
        assert!(atalho.lines().any(|atual| atual == linha), "{linha}");
    }
    assert!(atalho.contains(&alvo.exe().to_string_lossy().replace('\\', "\\\\\\\\")));
    for (tamanho, bytes) in ICONS {
        assert_eq!(std::fs::read(alvo.icon(tamanho)).unwrap(), bytes);
    }
    let falado = rodar(&home, None, &["--instalar"], "");
    assert_eq!(falado.codigo, 0);
    assert!(falado.texto.contains("Sistemas Lineares foi instalado."));
    std::fs::create_dir_all(&alvo.config_dir).unwrap();
    std::fs::write(alvo.preferences(), "(\"metodo\": \"cramer\")").unwrap();
    let cancelado = rodar(&home, None, &["--desinstalar"], "n\n");
    assert_eq!(cancelado.codigo, install::EXIT_CANCELLED);
    assert!(cancelado.texto.contains("Confirmar? [s/N]"));
    assert!(alvo.is_installed() && alvo.has_preferences());
    let removido = rodar(&home, None, &["--desinstalar"], "s\n");
    assert_eq!(removido.codigo, 0, "{}", removido.erros);
    assert!(removido.texto.contains("foi removido deste computador"));
    assert!(!alvo.is_installed());
    assert!(!alvo.has_preferences(), "as preferências também saem");
    let de_novo = rodar(&home, None, &["--instalar", "--silencioso"], "");
    assert_eq!(de_novo.codigo, 0);
    let sem_pergunta = rodar(&home, None, &["--desinstalar", "--silencioso"], "");
    assert_eq!(sem_pergunta.codigo, 0);
    assert!(sem_pergunta.texto.is_empty());
    assert!(!alvo.is_installed());
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn respeita_as_pastas_xdg() {
    let home = pasta("xdg");
    let dados = home.join("dados");
    let config = home.join("config");
    let resultado = rodar(
        &home,
        Some((&dados, &config)),
        &["--instalar", "--silencioso"],
        "",
    );
    assert_eq!(resultado.codigo, 0, "{}", resultado.erros);
    assert!(
        dados
            .join("applications")
            .join("sistemas-lineares.desktop")
            .is_file()
    );
    assert!(
        dados
            .join("icons")
            .join("hicolor")
            .join("256x256")
            .join("apps")
            .join("sistemas-lineares.png")
            .is_file()
    );
    assert!(
        home.join(".local")
            .join("bin")
            .join("sistemas-lineares")
            .is_file()
    );
    assert!(
        !home
            .join(".local")
            .join("share")
            .join("applications")
            .exists()
    );
    std::fs::create_dir_all(config.join("sistemas-lineares")).unwrap();
    let removido = rodar(
        &home,
        Some((&dados, &config)),
        &["--desinstalar", "--silencioso"],
        "",
    );
    assert_eq!(removido.codigo, 0);
    assert!(!config.join("sistemas-lineares").exists());
    assert!(
        !dados
            .join("applications")
            .join("sistemas-lineares.desktop")
            .exists()
    );
    let _ = std::fs::remove_dir_all(&home);
}
