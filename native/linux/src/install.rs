use std::ffi::OsString;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const APP_ID: &str = "sistemas-lineares";
pub const PRODUCT: &str = "Sistemas Lineares";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = "Resolva sistemas lineares 2×2 e 3×3 com cálculo exato";
pub const PREFERENCES_FILE: &str = "preferencias.ron";

pub const EXIT_OK: i32 = 0;
pub const EXIT_ERROR: i32 = 1;
pub const EXIT_CANCELLED: i32 = 2;

pub const ICONS: [(u32, &[u8]); 6] = [
    (16, include_bytes!("../assets/sistemas-lineares-16.png")),
    (32, include_bytes!("../assets/sistemas-lineares-32.png")),
    (48, include_bytes!("../assets/sistemas-lineares-48.png")),
    (64, include_bytes!("../assets/sistemas-lineares-64.png")),
    (128, include_bytes!("../assets/sistemas-lineares-128.png")),
    (256, include_bytes!("../assets/sistemas-lineares-256.png")),
];

pub const WINDOW_ICON: &[u8] = include_bytes!("../assets/sistemas-lineares-256.png");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Targets {
    pub bin_dir: PathBuf,
    pub applications_dir: PathBuf,
    pub icons_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Removal {
    Removed,
    NothingInstalled,
}

fn absolute(value: Option<OsString>) -> Option<PathBuf> {
    value
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

impl Targets {
    pub fn from_env(var: impl Fn(&str) -> Option<OsString>) -> Result<Targets, String> {
        let home = absolute(var("HOME")).ok_or_else(|| {
            "Não foi possível localizar a pasta pessoal: a variável HOME não está definida."
                .to_string()
        })?;
        let data_home =
            absolute(var("XDG_DATA_HOME")).unwrap_or_else(|| home.join(".local").join("share"));
        let config_home = absolute(var("XDG_CONFIG_HOME")).unwrap_or_else(|| home.join(".config"));
        Ok(Targets {
            bin_dir: home.join(".local").join("bin"),
            applications_dir: data_home.join("applications"),
            icons_dir: data_home.join("icons").join("hicolor"),
            config_dir: config_home.join(APP_ID),
            data_dir: data_home.join(APP_ID),
        })
    }

    pub fn for_user() -> Result<Targets, String> {
        Targets::from_env(|key| std::env::var_os(key))
    }

    pub fn exe(&self) -> PathBuf {
        self.bin_dir.join(APP_ID)
    }

    pub fn desktop(&self) -> PathBuf {
        self.applications_dir.join(format!("{APP_ID}.desktop"))
    }

    pub fn icon(&self, size: u32) -> PathBuf {
        self.icons_dir
            .join(format!("{size}x{size}"))
            .join("apps")
            .join(format!("{APP_ID}.png"))
    }

    pub fn preferences(&self) -> PathBuf {
        self.config_dir.join(PREFERENCES_FILE)
    }

    pub fn is_installed(&self) -> bool {
        self.exe().exists()
            || self.desktop().exists()
            || ICONS.iter().any(|(size, _)| self.icon(*size).exists())
    }

    pub fn has_preferences(&self) -> bool {
        self.config_dir.exists() || self.data_dir.exists()
    }
}

pub fn preferences_path() -> Option<PathBuf> {
    Targets::for_user()
        .ok()
        .map(|targets| targets.preferences())
}

fn quote_exec(path: &Path) -> String {
    let text = path.to_string_lossy();
    let plain = text
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "/._-+".contains(character));
    if plain {
        return text.into_owned();
    }
    let mut quoted = String::from("\"");
    for character in text.chars() {
        match character {
            '\\' => quoted.push_str("\\\\\\\\"),
            '"' | '`' | '$' => {
                quoted.push_str("\\\\");
                quoted.push(character);
            }
            '%' => quoted.push_str("%%"),
            '\n' | '\r' => {}
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

pub fn desktop_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Version=1.5\n\
         Name={PRODUCT}\n\
         GenericName=Resolvedor de sistemas lineares\n\
         Comment={DESCRIPTION}\n\
         Exec={}\n\
         Icon={APP_ID}\n\
         Terminal=false\n\
         Categories=Education;Science;Math;\n\
         Keywords=sistema;linear;equação;matriz;Gauss;Cramer;escalonamento;\n\
         StartupNotify=true\n\
         StartupWMClass={APP_ID}\n",
        quote_exec(exe)
    )
}

fn same_file(first: &Path, second: &Path) -> bool {
    match (std::fs::canonicalize(first), std::fs::canonicalize(second)) {
        (Ok(first), Ok(second)) => first == second,
        _ => false,
    }
}

fn write_atomic(
    target: &Path,
    write: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<(), String> {
    let directory = target
        .parent()
        .ok_or_else(|| format!("Caminho inválido: {}.", target.display()))?;
    std::fs::create_dir_all(directory).map_err(|error| {
        format!(
            "Não foi possível criar a pasta {}.\n\n{error}",
            directory.display()
        )
    })?;
    let name = target
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temporary = directory.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = write(&temporary).and_then(|()| std::fs::rename(&temporary, target));
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        return Err(format!(
            "Não foi possível gravar {}.\n\n{error}",
            target.display()
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> std::io::Result<()> {
    Ok(())
}

pub fn install(source: &Path, targets: &Targets) -> Result<(), String> {
    if !source.is_file() {
        return Err(format!(
            "Não foi possível encontrar o aplicativo em {}.",
            source.display()
        ));
    }
    let exe = targets.exe();
    if !same_file(source, &exe) {
        write_atomic(&exe, |temporary| {
            std::fs::copy(source, temporary)?;
            set_mode(temporary, 0o755)
        })?;
    }
    for (size, bytes) in ICONS {
        write_atomic(&targets.icon(size), |temporary| {
            std::fs::write(temporary, bytes)?;
            set_mode(temporary, 0o644)
        })?;
    }
    let entry = desktop_entry(&exe);
    write_atomic(&targets.desktop(), |temporary| {
        std::fs::write(temporary, entry.as_bytes())?;
        set_mode(temporary, 0o644)
    })
}

fn remove_file(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Não foi possível remover {}.\n\n{error}",
            path.display()
        )),
    }
}

fn remove_directory(path: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Não foi possível remover a pasta {}.\n\n{error}",
            path.display()
        )),
    }
}

pub fn uninstall(targets: &Targets) -> Result<Removal, String> {
    if !targets.is_installed() && !targets.has_preferences() {
        return Ok(Removal::NothingInstalled);
    }
    remove_file(&targets.desktop())?;
    for (size, _) in ICONS {
        remove_file(&targets.icon(size))?;
    }
    remove_directory(&targets.config_dir)?;
    remove_directory(&targets.data_dir)?;
    remove_file(&targets.exe())?;
    Ok(Removal::Removed)
}

fn run_quietly(program: &str, arguments: &[&OsString]) {
    let _ = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

pub fn refresh_caches(targets: &Targets) {
    if targets.applications_dir.exists() {
        run_quietly(
            "update-desktop-database",
            &[&targets.applications_dir.clone().into_os_string()],
        );
    }
    if targets.icons_dir.exists() {
        let icons = targets.icons_dir.clone().into_os_string();
        run_quietly(
            "gtk-update-icon-cache",
            &[&OsString::from("-f"), &OsString::from("-t"), &icons],
        );
    }
}

fn in_path(directory: &Path) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|entry| entry == directory || same_file(&entry, directory))
    })
}

pub fn run_install(silent: bool, output: &mut dyn Write, errors: &mut dyn Write) -> i32 {
    let result = Targets::for_user().and_then(|targets| {
        let source = std::env::current_exe().map_err(|error| {
            format!("Não foi possível localizar o próprio aplicativo.\n\n{error}")
        })?;
        install(&source, &targets).map(|()| targets)
    });
    match result {
        Ok(targets) => {
            refresh_caches(&targets);
            if !silent {
                let _ = writeln!(
                    output,
                    "{PRODUCT} foi instalado.\n\nAbra pelo menu de aplicativos ou com o comando {APP_ID}.\nPara remover, use {APP_ID} --desinstalar."
                );
                if !in_path(&targets.bin_dir) {
                    let _ = writeln!(
                        output,
                        "\nObservação: a pasta {} não está no PATH desta sessão. O atalho do menu funciona normalmente; para usar o comando no terminal, adicione essa pasta ao PATH.",
                        targets.bin_dir.display()
                    );
                }
            }
            EXIT_OK
        }
        Err(error) => {
            if !silent {
                let _ = writeln!(errors, "A instalação não foi concluída.\n\n{error}");
            }
            EXIT_ERROR
        }
    }
}

pub fn confirmed(answer: &str) -> bool {
    matches!(
        answer.trim().to_lowercase().as_str(),
        "s" | "sim" | "y" | "yes"
    )
}

pub fn run_uninstall_with(
    targets: Result<Targets, String>,
    silent: bool,
    input: &mut dyn BufRead,
    output: &mut dyn Write,
    errors: &mut dyn Write,
) -> i32 {
    let targets = match targets {
        Ok(targets) => targets,
        Err(error) => {
            if !silent {
                let _ = writeln!(errors, "{error}");
            }
            return EXIT_ERROR;
        }
    };
    if !targets.is_installed() && !targets.has_preferences() {
        if !silent {
            let _ = writeln!(output, "O {PRODUCT} não está instalado neste computador.");
        }
        return EXIT_OK;
    }
    if !silent {
        let _ = write!(
            output,
            "Remover o {PRODUCT} deste computador?\nO comando {APP_ID}, o atalho no menu de aplicativos, os ícones e as preferências salvas serão apagados.\nConfirmar? [s/N] "
        );
        let _ = output.flush();
        let mut answer = String::new();
        let read = input.read_line(&mut answer).unwrap_or(0);
        if read == 0 || !confirmed(&answer) {
            let _ = writeln!(output, "\nNada foi removido.");
            return EXIT_CANCELLED;
        }
    }
    match uninstall(&targets) {
        Ok(removal) => {
            refresh_caches(&targets);
            if !silent {
                let text = match removal {
                    Removal::Removed => format!("{PRODUCT} foi removido deste computador."),
                    Removal::NothingInstalled => {
                        format!("O {PRODUCT} não está instalado neste computador.")
                    }
                };
                let _ = writeln!(output, "{text}");
            }
            EXIT_OK
        }
        Err(error) => {
            if !silent {
                let _ = writeln!(errors, "A desinstalação não foi concluída.\n\n{error}");
            }
            EXIT_ERROR
        }
    }
}

pub fn run_uninstall(silent: bool) -> i32 {
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    run_uninstall_with(
        Targets::for_user(),
        silent,
        &mut input,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alvos(base: &Path) -> Targets {
        let base = base.to_path_buf();
        Targets::from_env(move |chave| match chave {
            "HOME" => Some(base.clone().into_os_string()),
            _ => None,
        })
        .expect("alvos")
    }

    fn raiz() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from("C:\\")
        } else {
            PathBuf::from("/")
        }
    }

    #[test]
    fn caminhos_xdg() {
        let home = raiz().join("home").join("ana");
        let dados = raiz().join("dados");
        let config = raiz().join("config");
        let pessoal = home.clone();
        let alvo = Targets::from_env(move |chave| match chave {
            "HOME" => Some(pessoal.clone().into_os_string()),
            _ => None,
        })
        .unwrap();
        let share = home.join(".local").join("share");
        assert_eq!(
            alvo.exe(),
            home.join(".local").join("bin").join("sistemas-lineares")
        );
        assert_eq!(
            alvo.desktop(),
            share.join("applications").join("sistemas-lineares.desktop")
        );
        assert_eq!(
            alvo.icon(48),
            share
                .join("icons")
                .join("hicolor")
                .join("48x48")
                .join("apps")
                .join("sistemas-lineares.png")
        );
        assert_eq!(
            alvo.preferences(),
            home.join(".config")
                .join("sistemas-lineares")
                .join("preferencias.ron")
        );
        let (pessoal, xdg_dados, xdg_config) = (home.clone(), dados.clone(), config.clone());
        let xdg = Targets::from_env(move |chave| match chave {
            "HOME" => Some(pessoal.clone().into_os_string()),
            "XDG_DATA_HOME" => Some(xdg_dados.clone().into_os_string()),
            "XDG_CONFIG_HOME" => Some(xdg_config.clone().into_os_string()),
            _ => None,
        })
        .unwrap();
        assert_eq!(xdg.applications_dir, dados.join("applications"));
        assert_eq!(xdg.icons_dir, dados.join("icons").join("hicolor"));
        assert_eq!(xdg.config_dir, config.join("sistemas-lineares"));
        assert_eq!(xdg.data_dir, dados.join("sistemas-lineares"));
        assert_eq!(xdg.bin_dir, home.join(".local").join("bin"));
        let pessoal = home.clone();
        let relativo = Targets::from_env(move |chave| match chave {
            "HOME" => Some(pessoal.clone().into_os_string()),
            "XDG_DATA_HOME" => Some(OsString::from("relativo")),
            _ => None,
        })
        .unwrap();
        assert_eq!(
            relativo.applications_dir,
            share.join("applications"),
            "XDG relativo é ignorado"
        );
        assert!(Targets::from_env(|_| None).is_err());
        assert!(
            Targets::from_env(|chave| (chave == "HOME").then(|| OsString::from("relativo")))
                .is_err()
        );
    }

    #[test]
    fn atalho_do_menu() {
        let texto = desktop_entry(Path::new("/home/ana/.local/bin/sistemas-lineares"));
        for linha in [
            "[Desktop Entry]",
            "Type=Application",
            "Name=Sistemas Lineares",
            "Comment=Resolva sistemas lineares 2×2 e 3×3 com cálculo exato",
            "Exec=/home/ana/.local/bin/sistemas-lineares",
            "Icon=sistemas-lineares",
            "Terminal=false",
            "Categories=Education;Science;Math;",
            "StartupWMClass=sistemas-lineares",
        ] {
            assert!(texto.lines().any(|atual| atual == linha), "{linha}");
        }
        assert_eq!(
            quote_exec(Path::new("/home/Ana Maria/bin/app")),
            "\"/home/Ana Maria/bin/app\""
        );
        assert_eq!(
            quote_exec(Path::new("/home/a$b/100%/app")),
            "\"/home/a\\\\$b/100%%/app\""
        );
    }

    #[test]
    fn respostas_de_confirmacao() {
        for sim in ["s", "S", "sim", " Sim\n", "y", "yes"] {
            assert!(confirmed(sim), "{sim:?}");
        }
        for nao in ["", "n", "nao", "não", "talvez"] {
            assert!(!confirmed(nao), "{nao:?}");
        }
    }

    #[test]
    fn icones_embutidos_sao_png() {
        for (tamanho, bytes) in ICONS {
            assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n", "{tamanho}");
            let largura = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
            let altura = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
            assert_eq!((largura, altura), (tamanho, tamanho));
        }
        assert_eq!(WINDOW_ICON, ICONS[5].1);
    }

    fn pasta_temporaria(nome: &str) -> PathBuf {
        let pasta = std::env::temp_dir().join(format!(
            "sl-linux-{nome}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |tempo| tempo.as_nanos())
        ));
        std::fs::create_dir_all(&pasta).unwrap();
        pasta
    }

    #[test]
    fn instala_e_desinstala_num_home_temporario() {
        let home = pasta_temporaria("ciclo");
        let alvo = alvos(&home);
        let origem = home.join("origem-do-binario");
        std::fs::write(&origem, b"binario de teste").unwrap();
        assert!(!alvo.is_installed());
        assert_eq!(uninstall(&alvo), Ok(Removal::NothingInstalled));
        install(&origem, &alvo).expect("instalação");
        assert!(alvo.is_installed());
        assert_eq!(std::fs::read(alvo.exe()).unwrap(), b"binario de teste");
        let atalho = std::fs::read_to_string(alvo.desktop()).unwrap();
        assert!(atalho.contains(&format!("Exec={}", quote_exec(&alvo.exe()))));
        for (tamanho, bytes) in ICONS {
            assert_eq!(std::fs::read(alvo.icon(tamanho)).unwrap(), bytes);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let modo = std::fs::metadata(alvo.exe()).unwrap().permissions().mode();
            assert_eq!(modo & 0o777, 0o755);
        }
        install(&origem, &alvo).expect("reinstalar por cima");
        std::fs::create_dir_all(&alvo.config_dir).unwrap();
        std::fs::write(alvo.preferences(), b"{}").unwrap();
        assert_eq!(uninstall(&alvo), Ok(Removal::Removed));
        assert!(!alvo.is_installed());
        assert!(!alvo.has_preferences());
        assert!(!alvo.exe().exists());
        assert_eq!(uninstall(&alvo), Ok(Removal::NothingInstalled));
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn instalar_sem_origem_falha() {
        let home = pasta_temporaria("sem-origem");
        let alvo = alvos(&home);
        let erro = install(&home.join("nao-existe"), &alvo).unwrap_err();
        assert!(
            erro.contains("Não foi possível encontrar o aplicativo"),
            "{erro}"
        );
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn desinstalar_pergunta_e_respeita_a_resposta() {
        let home = pasta_temporaria("pergunta");
        let alvo = alvos(&home);
        let origem = home.join("origem");
        std::fs::write(&origem, b"x").unwrap();
        install(&origem, &alvo).unwrap();
        let mut saida = Vec::new();
        let mut erros = Vec::new();
        let codigo = run_uninstall_with(
            Ok(alvo.clone()),
            false,
            &mut "n\n".as_bytes(),
            &mut saida,
            &mut erros,
        );
        assert_eq!(codigo, EXIT_CANCELLED);
        assert!(alvo.is_installed());
        let texto = String::from_utf8(saida).unwrap();
        assert!(texto.contains("Remover o Sistemas Lineares deste computador?"));
        assert!(texto.contains("Nada foi removido."));
        let codigo = run_uninstall_with(
            Ok(alvo.clone()),
            false,
            &mut "".as_bytes(),
            &mut Vec::new(),
            &mut Vec::new(),
        );
        assert_eq!(codigo, EXIT_CANCELLED, "sem resposta, cancela");
        let mut saida = Vec::new();
        let codigo = run_uninstall_with(
            Ok(alvo.clone()),
            false,
            &mut "sim\n".as_bytes(),
            &mut saida,
            &mut Vec::new(),
        );
        assert_eq!(codigo, EXIT_OK);
        assert!(!alvo.is_installed());
        assert!(
            String::from_utf8(saida)
                .unwrap()
                .contains("Sistemas Lineares foi removido deste computador.")
        );
        let mut saida = Vec::new();
        let codigo = run_uninstall_with(
            Ok(alvo.clone()),
            false,
            &mut "".as_bytes(),
            &mut saida,
            &mut Vec::new(),
        );
        assert_eq!(codigo, EXIT_OK);
        assert!(
            String::from_utf8(saida)
                .unwrap()
                .contains("não está instalado neste computador")
        );
        install(&origem, &alvo).unwrap();
        let mut saida = Vec::new();
        let codigo = run_uninstall_with(
            Ok(alvo.clone()),
            true,
            &mut "".as_bytes(),
            &mut saida,
            &mut Vec::new(),
        );
        assert_eq!(codigo, EXIT_OK, "silencioso não pergunta");
        assert!(saida.is_empty());
        assert!(!alvo.is_installed());
        let mut erros = Vec::new();
        let codigo = run_uninstall_with(
            Err("sem HOME".to_string()),
            false,
            &mut "".as_bytes(),
            &mut Vec::new(),
            &mut erros,
        );
        assert_eq!(codigo, EXIT_ERROR);
        assert!(String::from_utf8(erros).unwrap().contains("sem HOME"));
        let _ = std::fs::remove_dir_all(&home);
    }
}
