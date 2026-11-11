use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sistemas_lineares::install::{self, Removal, Targets};
use sistemas_lineares::registry::{self, Key};
use sistemas_lineares::settings::{self, Bounds, Placement, Saved};
use sistemas_lineares::{Launch, create, message_loop};
use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, IPersistFile,
    STGM_READ,
};
use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
use windows::Win32::UI::WindowsAndMessaging::IsWindow;
use windows::core::{HSTRING, Interface};

const RAIZ: &str = r"Software\Lucas Lima\Testes Sistemas Lineares";

struct Ambiente {
    pasta: PathBuf,
    alvos: Targets,
}

impl Ambiente {
    fn novo(nome: &str) -> Ambiente {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |tempo| tempo.as_nanos());
        let marca = format!("{nome}-{}-{nanos}", std::process::id());
        let pasta = std::env::temp_dir().join(format!("sistemas-lineares-teste-{marca}"));
        let alvos = Targets {
            install_dir: pasta.join("Programs").join("Sistemas Lineares"),
            start_menu_dir: pasta.join("Menu Iniciar").join("Programas"),
            uninstall_key: format!(r"{RAIZ}\{marca}\Uninstall\SistemasLineares"),
            settings_key: format!(r"{RAIZ}\{marca}\Preferencias"),
        };
        assert!(
            alvos.install_dir.starts_with(std::env::temp_dir()),
            "testes só usam a pasta temporária"
        );
        assert!(
            alvos.uninstall_key.starts_with(RAIZ) && alvos.settings_key.starts_with(RAIZ),
            "testes só usam a subchave de teste"
        );
        Ambiente { pasta, alvos }
    }
}

impl Drop for Ambiente {
    fn drop(&mut self) {
        for _ in 0..50 {
            if std::fs::remove_dir_all(&self.pasta).is_ok() || !self.pasta.exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if let Some(marca) = registry::parent(&self.alvos.settings_key) {
            registry::delete_tree(marca);
        }
        registry::delete_if_empty(RAIZ);
        registry::delete_if_empty(r"Software\Lucas Lima");
    }
}

fn origem() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_SistemasLineares"))
}

fn com() {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
    }
}

fn texto(buffer: &[u16]) -> String {
    let fim = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..fim])
}

fn ler_atalho(atalho: &Path) -> (String, String, String, String, i32) {
    unsafe {
        let link: IShellLinkW =
            CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).expect("IShellLinkW");
        let arquivo: IPersistFile = link.cast().expect("IPersistFile");
        arquivo
            .Load(&HSTRING::from(atalho.as_os_str()), STGM_READ)
            .expect("abrir o atalho");
        let mut destino = [0u16; 1024];
        let mut dados = WIN32_FIND_DATAW::default();
        link.GetPath(&mut destino, &mut dados, 0).expect("destino");
        let mut pasta = [0u16; 1024];
        link.GetWorkingDirectory(&mut pasta)
            .expect("pasta de trabalho");
        let mut descricao = [0u16; 1024];
        link.GetDescription(&mut descricao).expect("descrição");
        let mut icone = [0u16; 1024];
        let mut indice = -1;
        link.GetIconLocation(&mut icone, &mut indice)
            .expect("ícone");
        (
            texto(&destino),
            texto(&pasta),
            texto(&descricao),
            texto(&icone),
            indice,
        )
    }
}

#[test]
fn instala_reinstala_e_desinstala_em_pasta_de_teste() {
    com();
    let ambiente = Ambiente::novo("ciclo");
    let alvos = &ambiente.alvos;
    let fonte = origem();
    let tamanho = std::fs::metadata(&fonte).expect("exe compilado").len();
    assert!(!alvos.is_installed());
    install::install(&fonte, alvos).expect("instalação de teste");
    let exe = alvos.exe();
    assert!(exe.is_file());
    assert_eq!(
        std::fs::metadata(&exe).map(|meta| meta.len()).unwrap_or(0),
        tamanho
    );
    let atalho = alvos.shortcut();
    assert!(atalho.is_file(), "atalho criado em {}", atalho.display());
    assert_eq!(
        atalho.file_name().and_then(|nome| nome.to_str()),
        Some("Sistemas Lineares.lnk")
    );
    let (destino, pasta, descricao, icone, indice) = ler_atalho(&atalho);
    assert!(
        install::same_path(Path::new(&destino), &exe),
        "destino do atalho: {destino}"
    );
    assert!(
        install::same_path(Path::new(&pasta), &alvos.install_dir),
        "pasta de trabalho: {pasta}"
    );
    assert_eq!(
        descricao,
        "Resolva sistemas lineares 2×2 e 3×3 com cálculo exato"
    );
    assert!(install::same_path(Path::new(&icone), &exe));
    assert_eq!(indice, 0);
    let exe_texto = exe.to_string_lossy().to_string();
    let chave = Key::open(&alvos.uninstall_key).expect("chave de desinstalação");
    let esperado = [
        ("DisplayName", "Sistemas Lineares".to_string()),
        ("DisplayVersion", "4.0.0".to_string()),
        ("Publisher", "Lucas Lima".to_string()),
        ("Comments", "Feito por Lucas Lima · RA 185964".to_string()),
        ("DisplayIcon", format!("{exe_texto},0")),
        (
            "InstallLocation",
            alvos.install_dir.to_string_lossy().to_string(),
        ),
        ("UninstallString", format!("\"{exe_texto}\" --desinstalar")),
        (
            "QuietUninstallString",
            format!("\"{exe_texto}\" --desinstalar --silencioso"),
        ),
        ("URLInfoAbout", "https://lucas-lima.xyz/".to_string()),
    ];
    for (nome, valor) in &esperado {
        assert_eq!(
            chave.string(nome).as_deref(),
            Some(valor.as_str()),
            "valor {nome}"
        );
    }
    assert_eq!(chave.dword("NoModify"), Some(1));
    assert_eq!(chave.dword("NoRepair"), Some(1));
    assert_eq!(
        chave.dword("EstimatedSize"),
        Some(tamanho.div_ceil(1024) as u32)
    );
    assert_eq!(chave.string("InstallDate").map(|data| data.len()), Some(8));
    drop(chave);
    assert!(alvos.is_installed());

    std::fs::write(&exe, b"versao antiga").expect("simula versão antiga");
    install::install(&fonte, alvos).expect("reinstalar por cima");
    assert_eq!(
        std::fs::metadata(&exe).map(|meta| meta.len()).unwrap_or(0),
        tamanho,
        "reinstalar atualiza o exe"
    );
    assert!(atalho.is_file());

    assert!(settings::save(
        &alvos.settings_key,
        &Saved {
            placement: Some(Placement {
                bounds: Bounds {
                    left: 1,
                    top: 2,
                    width: 800,
                    height: 600
                },
                maximized: false,
                dpi: 96
            }),
            dimension: Some(3)
        }
    ));
    assert!(registry::exists(&alvos.settings_key));
    let rodando = std::env::current_exe().expect("exe do teste");
    assert_eq!(install::uninstall(alvos, &rodando), Ok(Removal::Removed));
    assert!(!alvos.install_dir.exists(), "pasta removida");
    assert!(!atalho.exists(), "atalho removido");
    assert!(
        !registry::exists(&alvos.uninstall_key),
        "chave de desinstalação removida"
    );
    assert!(
        !registry::exists(&alvos.settings_key),
        "preferências removidas"
    );
    assert_eq!(
        install::uninstall(alvos, &rodando),
        Ok(Removal::NothingInstalled)
    );
}

#[test]
fn fecha_a_copia_aberta_antes_de_atualizar() {
    let exe = std::env::current_exe().expect("exe do teste");
    let thread = std::thread::spawn(|| {
        let launch = Launch {
            quit_on_close: true,
            ..Launch::background((-4000, 0), (1160, 810))
        };
        let handles = create(&launch).expect("janela");
        let _ = handles.main.0 as isize;
        message_loop(handles.main)
    });
    let limite = Instant::now() + Duration::from_secs(10);
    let mut janelas = Vec::new();
    while janelas.is_empty() && Instant::now() < limite {
        janelas = install::windows_of(&exe);
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !janelas.is_empty(),
        "janela da cópia em execução encontrada pela classe e pelo caminho do exe"
    );
    assert!(install::windows_of(Path::new(r"C:\nao\existe\SistemasLineares.exe")).is_empty());
    install::close_running(&exe, install::CLOSE_TIMEOUT).expect("fecha com WM_CLOSE");
    for (hwnd, _) in &janelas {
        assert!(
            !unsafe { IsWindow(Some(*hwnd)).as_bool() },
            "janela fechada"
        );
    }
    assert_eq!(thread.join().ok(), Some(0));
}

#[test]
fn caminhos_do_usuario_e_codigos_de_saida() {
    com();
    let alvos = Targets::for_user().expect("pastas do usuário");
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .expect("LOCALAPPDATA");
    let roaming = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .expect("APPDATA");
    assert!(
        install::same_path(
            &alvos.install_dir,
            &local.join("Programs").join("Sistemas Lineares")
        ),
        "{}",
        alvos.install_dir.display()
    );
    assert!(
        install::same_path(
            &alvos.start_menu_dir,
            &roaming.join(r"Microsoft\Windows\Start Menu\Programs")
        ),
        "{}",
        alvos.start_menu_dir.display()
    );
    assert_eq!(
        alvos.uninstall_key,
        r"Software\Microsoft\Windows\CurrentVersion\Uninstall\SistemasLineares"
    );
    assert_eq!(alvos.settings_key, r"Software\Lucas Lima\Sistemas Lineares");
    assert_eq!(
        alvos.exe().file_name().and_then(|nome| nome.to_str()),
        Some("SistemasLineares.exe")
    );
    assert_eq!(
        (
            install::EXIT_OK,
            install::EXIT_ERROR,
            install::EXIT_CANCELLED
        ),
        (0, 1, 2)
    );
    assert_eq!(install::VERSION, "4.0.0");
}

#[test]
fn exe_em_uso_e_apagado_pelo_processo_auxiliar_oculto() {
    use std::os::windows::fs::OpenOptionsExt;
    com();
    let ambiente = Ambiente::novo("auxiliar");
    let alvos = &ambiente.alvos;
    install::install(&origem(), alvos).expect("instalação de teste");
    let exe = alvos.exe();
    let travado = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&exe)
        .expect("simula o exe em execução");
    assert!(
        std::fs::remove_file(&exe).is_err(),
        "o exe em uso não pode ser apagado diretamente"
    );
    assert_eq!(install::uninstall(alvos, &exe), Ok(Removal::Removed));
    assert!(!alvos.shortcut().exists() && !registry::exists(&alvos.uninstall_key));
    std::thread::sleep(Duration::from_millis(1500));
    assert!(
        alvos.install_dir.exists() && exe.exists(),
        "a pasta continua enquanto o exe está em uso"
    );
    drop(travado);
    let limite = Instant::now() + Duration::from_secs(20);
    while alvos.install_dir.exists() && Instant::now() < limite {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        !alvos.install_dir.exists(),
        "o processo auxiliar oculto apagou a pasta depois que o exe foi liberado"
    );
    std::fs::create_dir_all(&alvos.install_dir).expect("pasta recriada");
    let marca = alvos.install_dir.join("reinstalado.txt");
    std::fs::write(&marca, "nova instalação").expect("marca");
    std::thread::sleep(Duration::from_millis(2500));
    assert!(
        marca.exists(),
        "o processo auxiliar sai assim que apaga e não toca numa instalação nova"
    );
}

#[test]
fn exe_em_execucao_sai_da_pasta_na_hora_e_reinstalar_logo_depois_funciona() {
    use std::os::windows::process::CommandExt;
    com();
    let ambiente = Ambiente::novo("em-execucao");
    let alvos = &ambiente.alvos;
    install::install(&origem(), alvos).expect("instalação de teste");
    let sistema =
        std::env::var_os("SystemRoot").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
    let em_uso = alvos.install_dir.join("ajudante-em-uso.exe");
    std::fs::copy(sistema.join(r"System32\PING.EXE"), &em_uso).expect("cópia do ping");
    let mut processo = std::process::Command::new(&em_uso)
        .args(["-n", "4", "127.0.0.1"])
        .stdout(std::process::Stdio::null())
        .creation_flags(0x0800_0000)
        .spawn()
        .expect("processo em execução dentro da pasta instalada");
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(install::uninstall(alvos, &em_uso), Ok(Removal::Removed));
    assert!(
        !alvos.install_dir.exists(),
        "o exe em execução sai da pasta e a pasta some na hora"
    );
    install::install(&origem(), alvos).expect("reinstalar logo depois");
    std::thread::sleep(Duration::from_millis(2500));
    assert!(
        alvos.exe().exists(),
        "a nova instalação não é apagada pelo processo auxiliar"
    );
    let _ = processo.wait();
    let prefixo = format!("SistemasLineares-removido-{}-", std::process::id());
    let pendentes = || {
        std::fs::read_dir(std::env::temp_dir())
            .map(|entradas| {
                entradas
                    .filter_map(Result::ok)
                    .filter(|entrada| entrada.file_name().to_string_lossy().starts_with(&prefixo))
                    .count()
            })
            .unwrap_or(0)
    };
    let limite = Instant::now() + Duration::from_secs(20);
    while pendentes() > 0 && Instant::now() < limite {
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(
        pendentes(),
        0,
        "o exe estacionado na pasta temporária é apagado quando o processo termina"
    );
}
