use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, WAIT_TIMEOUT, WPARAM};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree, IPersistFile,
};
use windows::Win32::System::SystemInformation::{GetLocalTime, GetSystemDirectoryW};
use windows::Win32::System::Threading::{
    CREATE_NO_WINDOW, CreateProcessW, GetCurrentProcessId, OpenProcess, PROCESS_INFORMATION,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    QueryFullProcessImageNameW, STARTUPINFOW, WaitForSingleObject,
};
use windows::Win32::UI::Shell::{
    FOLDERID_Programs, FOLDERID_UserProgramFiles, IShellLinkW, KF_FLAG_DONT_VERIFY, SHCNE_CREATE,
    SHCNE_DELETE, SHCNF_PATHW, SHChangeNotify, SHGetKnownFolderPath, ShellExecuteW, ShellLink,
};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GetWindowThreadProcessId, IDYES, IsWindow, MB_ICONERROR, MB_ICONINFORMATION,
    MB_ICONQUESTION, MB_OK, MB_SETFOREGROUND, MB_YESNO, MESSAGEBOX_STYLE, MessageBoxW,
    PostMessageW, SW_SHOWNORMAL, WM_CLOSE,
};
use windows::core::{GUID, HSTRING, Interface, PCWSTR, PWSTR, w};

use crate::model;
use crate::registry::{self, Key};
use crate::settings;
use crate::window::MAIN_CLASS_NAME;

pub const PRODUCT: &str = "Sistemas Lineares";
pub const EXE_NAME: &str = "SistemasLineares.exe";
pub const SHORTCUT_NAME: &str = "Sistemas Lineares.lnk";
pub const UNINSTALL_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Uninstall\SistemasLineares";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PUBLISHER: &str = "Lucas Lima";
pub const DESCRIPTION: &str = "Resolva sistemas lineares 2×2 e 3×3 com cálculo exato";
pub const ABOUT_URL: &str = "https://lucas-lima.xyz/";
pub const CLOSE_TIMEOUT: Duration = Duration::from_secs(5);

pub const EXIT_OK: i32 = 0;
pub const EXIT_ERROR: i32 = 1;
pub const EXIT_CANCELLED: i32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Targets {
    pub install_dir: PathBuf,
    pub start_menu_dir: PathBuf,
    pub uninstall_key: String,
    pub settings_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Removal {
    Removed,
    NothingInstalled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Install,
    Uninstall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arguments {
    pub command: Option<Command>,
    pub silent: bool,
}

pub fn parse_arguments<I: IntoIterator<Item = String>>(arguments: I) -> Arguments {
    let mut parsed = Arguments {
        command: None,
        silent: false,
    };
    for argument in arguments {
        match argument.trim().to_lowercase().as_str() {
            "--instalar" if parsed.command.is_none() => parsed.command = Some(Command::Install),
            "--desinstalar" if parsed.command.is_none() => {
                parsed.command = Some(Command::Uninstall)
            }
            "--silencioso" => parsed.silent = true,
            _ => {}
        }
    }
    parsed
}

fn known_folder(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let path = SHGetKnownFolderPath(id, KF_FLAG_DONT_VERIFY, None).ok()?;
        let text = path.to_string().ok();
        CoTaskMemFree(Some(path.0 as *const _));
        text.filter(|value| !value.is_empty()).map(PathBuf::from)
    }
}

impl Targets {
    pub fn for_user() -> Result<Targets, String> {
        let programs = known_folder(&FOLDERID_UserProgramFiles).or_else(|| {
            std::env::var_os("LOCALAPPDATA").map(|base| PathBuf::from(base).join("Programs"))
        });
        let start_menu = known_folder(&FOLDERID_Programs).or_else(|| {
            std::env::var_os("APPDATA")
                .map(|base| PathBuf::from(base).join(r"Microsoft\Windows\Start Menu\Programs"))
        });
        match (programs, start_menu) {
            (Some(programs), Some(start_menu)) => Ok(Targets {
                install_dir: programs.join(PRODUCT),
                start_menu_dir: start_menu,
                uninstall_key: UNINSTALL_KEY.to_string(),
                settings_key: settings::KEY.to_string(),
            }),
            _ => Err("Não foi possível localizar as pastas do usuário no Windows.".to_string()),
        }
    }

    pub fn exe(&self) -> PathBuf {
        self.install_dir.join(EXE_NAME)
    }

    pub fn shortcut(&self) -> PathBuf {
        self.start_menu_dir.join(SHORTCUT_NAME)
    }

    pub fn is_installed(&self) -> bool {
        self.install_dir.exists()
            || self.shortcut().exists()
            || registry::exists(&self.uninstall_key)
    }
}

fn comparable(path: &Path) -> String {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let text = resolved.to_string_lossy().replace('/', "\\");
    text.strip_prefix(r"\\?\").unwrap_or(&text).to_lowercase()
}

pub fn same_path(first: &Path, second: &Path) -> bool {
    comparable(first) == comparable(second)
}

fn process_image(pid: u32) -> Option<PathBuf> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = vec![0u16; 32_768];
        let mut size = buffer.len() as u32;
        let queried = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        )
        .is_ok();
        let _ = CloseHandle(process);
        queried.then(|| PathBuf::from(String::from_utf16_lossy(&buffer[..size as usize])))
    }
}

pub fn windows_of(exe: &Path) -> Vec<(HWND, u32)> {
    let class = HSTRING::from(MAIN_CLASS_NAME);
    let mut found = Vec::new();
    let mut previous: Option<HWND> = None;
    for _ in 0..4096 {
        let Ok(hwnd) = (unsafe { FindWindowExW(None, previous, &class, PCWSTR::null()) }) else {
            break;
        };
        if hwnd.is_invalid() {
            break;
        }
        previous = Some(hwnd);
        let mut pid = 0u32;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if pid != 0 && process_image(pid).is_some_and(|image| same_path(&image, exe)) {
            found.push((hwnd, pid));
        }
    }
    found
}

struct Process(HANDLE);

impl Drop for Process {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub fn close_running(exe: &Path, timeout: Duration) -> Result<(), String> {
    let windows = windows_of(exe);
    if windows.is_empty() {
        return Ok(());
    }
    let own = unsafe { GetCurrentProcessId() };
    let mut pids: Vec<u32> = windows
        .iter()
        .map(|(_, pid)| *pid)
        .filter(|pid| *pid != own)
        .collect();
    pids.sort_unstable();
    pids.dedup();
    let processes: Vec<Process> = pids
        .into_iter()
        .filter_map(|pid| {
            unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }
                .ok()
                .map(Process)
        })
        .collect();
    for (hwnd, _) in &windows {
        unsafe {
            let _ = PostMessageW(Some(*hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
    let deadline = Instant::now() + timeout;
    loop {
        let windows_open = windows
            .iter()
            .any(|(hwnd, _)| unsafe { IsWindow(Some(*hwnd)).as_bool() });
        let processes_alive = processes
            .iter()
            .any(|process| unsafe { WaitForSingleObject(process.0, 0) } == WAIT_TIMEOUT);
        if !windows_open && !processes_alive {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("O Sistemas Lineares instalado continua aberto. Feche a janela dele e tente de novo.".to_string());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn copy_with_retry(source: &Path, target: &Path) -> Result<u64, String> {
    if target.exists() && same_path(source, target) {
        return std::fs::metadata(target)
            .map(|metadata| metadata.len())
            .map_err(|error| format!("Não foi possível ler {}.\n\n{error}", target.display()));
    }
    let mut last = None;
    for attempt in 0..40 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(125));
        }
        match std::fs::copy(source, target) {
            Ok(bytes) => return Ok(bytes),
            Err(error) => last = Some(error),
        }
    }
    Err(format!(
        "Não foi possível copiar o aplicativo para {}.\n\n{}",
        target.display(),
        last.map(|error| error.to_string()).unwrap_or_default()
    ))
}

fn create_shortcut(
    shortcut: &Path,
    target: &Path,
    working_dir: &Path,
) -> windows::core::Result<()> {
    unsafe {
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        link.SetPath(&HSTRING::from(target.as_os_str()))?;
        link.SetWorkingDirectory(&HSTRING::from(working_dir.as_os_str()))?;
        link.SetIconLocation(&HSTRING::from(target.as_os_str()), 0)?;
        link.SetDescription(&HSTRING::from(DESCRIPTION))?;
        let file: IPersistFile = link.cast()?;
        file.Save(&HSTRING::from(shortcut.as_os_str()), true)
    }
}

fn notify_shell(event: windows::Win32::UI::Shell::SHCNE_ID, path: &Path) {
    let wide = HSTRING::from(path.as_os_str());
    unsafe { SHChangeNotify(event, SHCNF_PATHW, Some(wide.as_ptr().cast()), None) };
}

fn install_date() -> String {
    let now = unsafe { GetLocalTime() };
    format!("{:04}{:02}{:02}", now.wYear, now.wMonth, now.wDay)
}

pub fn register(targets: &Targets, size_bytes: u64) -> Result<(), String> {
    let exe = targets.exe();
    let exe_text = exe.to_string_lossy();
    let failure =
        || "Não foi possível registrar o aplicativo em Aplicativos instalados.".to_string();
    let key = Key::create(&targets.uninstall_key).ok_or_else(failure)?;
    let strings = [
        ("DisplayName", PRODUCT.to_string()),
        ("DisplayVersion", VERSION.to_string()),
        ("Publisher", PUBLISHER.to_string()),
        ("Comments", model::AUTHOR_LABEL.to_string()),
        ("DisplayIcon", format!("{exe_text},0")),
        (
            "InstallLocation",
            targets.install_dir.to_string_lossy().into_owned(),
        ),
        ("UninstallString", format!("\"{exe_text}\" --desinstalar")),
        (
            "QuietUninstallString",
            format!("\"{exe_text}\" --desinstalar --silencioso"),
        ),
        ("URLInfoAbout", ABOUT_URL.to_string()),
        ("InstallDate", install_date()),
    ];
    let mut written = strings
        .iter()
        .all(|(name, value)| key.set_string(name, value));
    written &= key.set_dword("NoModify", 1);
    written &= key.set_dword("NoRepair", 1);
    written &= key.set_dword(
        "EstimatedSize",
        size_bytes.div_ceil(1024).min(u64::from(u32::MAX)) as u32,
    );
    if written { Ok(()) } else { Err(failure()) }
}

pub fn install(source: &Path, targets: &Targets) -> Result<(), String> {
    if !source.is_file() {
        return Err(format!(
            "Não foi possível encontrar o aplicativo em {}.",
            source.display()
        ));
    }
    let exe = targets.exe();
    close_running(&exe, CLOSE_TIMEOUT)?;
    std::fs::create_dir_all(&targets.install_dir).map_err(|error| {
        format!(
            "Não foi possível criar a pasta {}.\n\n{error}",
            targets.install_dir.display()
        )
    })?;
    let size = copy_with_retry(source, &exe)?;
    std::fs::create_dir_all(&targets.start_menu_dir)
        .map_err(|error| format!("Não foi possível acessar a pasta do menu Iniciar.\n\n{error}"))?;
    let shortcut = targets.shortcut();
    create_shortcut(&shortcut, &exe, &targets.install_dir).map_err(|error| {
        format!(
            "Não foi possível criar o atalho no menu Iniciar.\n\n{}",
            error.message()
        )
    })?;
    notify_shell(SHCNE_CREATE, &shortcut);
    register(targets, size)
}

fn system_tool(name: &str) -> PathBuf {
    let mut buffer = vec![0u16; 512];
    let length = unsafe { GetSystemDirectoryW(Some(&mut buffer)) } as usize;
    let directory = if length > 0 && length < buffer.len() {
        PathBuf::from(String::from_utf16_lossy(&buffer[..length]))
    } else {
        PathBuf::from(r"C:\Windows\System32")
    };
    directory.join(name)
}

pub fn removal_command(target: &Path, directory: bool) -> String {
    let path = target.to_string_lossy();
    let ping = system_tool("PING.EXE");
    let remove = if directory {
        format!("rd /s /q \"{path}\"")
    } else {
        format!("del /f /q \"{path}\"")
    };
    format!(
        "cmd.exe /d /q /s /c \"for /l %n in (1,1,600) do (if exist \"{path}\" ({remove} >nul 2>&1 & if exist \"{path}\" (\"{}\" -n 2 127.0.0.1 >nul) else exit) else exit)\"",
        ping.to_string_lossy()
    )
}

fn schedule_removal(target: &Path, directory: bool) -> bool {
    let shell = HSTRING::from(system_tool("cmd.exe").as_os_str());
    let mut command: Vec<u16> = removal_command(target, directory)
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let working = HSTRING::from(system_tool("").as_os_str());
    let startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut information = PROCESS_INFORMATION::default();
    let created = unsafe {
        CreateProcessW(
            &shell,
            Some(PWSTR(command.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_NO_WINDOW,
            None,
            &working,
            &startup,
            &mut information,
        )
    }
    .is_ok();
    if created {
        unsafe {
            let _ = CloseHandle(information.hThread);
            let _ = CloseHandle(information.hProcess);
        }
    }
    created
}

fn remove_now(directory: &Path) -> bool {
    for attempt in 0..20 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_millis(100));
        }
        if std::fs::remove_dir_all(directory).is_ok() || !directory.exists() {
            return true;
        }
    }
    false
}

fn park(running_exe: &Path) -> Option<PathBuf> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let parked = std::env::temp_dir().join(format!(
        "SistemasLineares-removido-{}-{stamp}.exe",
        std::process::id()
    ));
    std::fs::rename(running_exe, &parked).ok().map(|()| parked)
}

fn remove_directory(install_dir: &Path, running_exe: &Path) -> Result<(), String> {
    if !install_dir.exists() {
        return Ok(());
    }
    let running_inside = std::fs::canonicalize(running_exe)
        .ok()
        .zip(std::fs::canonicalize(install_dir).ok())
        .is_some_and(|(exe, dir)| exe.starts_with(dir));
    if running_inside {
        if let Some(parked) = park(running_exe) {
            let removed = remove_now(install_dir);
            schedule_removal(&parked, false);
            if removed {
                return Ok(());
            }
        }
    } else if remove_now(install_dir) {
        return Ok(());
    }
    if schedule_removal(install_dir, true) {
        Ok(())
    } else {
        Err(format!(
            "Não foi possível remover a pasta {}.",
            install_dir.display()
        ))
    }
}

pub fn uninstall(targets: &Targets, running_exe: &Path) -> Result<Removal, String> {
    let had_settings = registry::exists(&targets.settings_key);
    if !targets.is_installed() && !had_settings {
        return Ok(Removal::NothingInstalled);
    }
    close_running(&targets.exe(), CLOSE_TIMEOUT)?;
    let shortcut = targets.shortcut();
    if shortcut.exists() {
        std::fs::remove_file(&shortcut).map_err(|error| {
            format!("Não foi possível remover o atalho do menu Iniciar.\n\n{error}")
        })?;
        notify_shell(SHCNE_DELETE, &shortcut);
    }
    if !registry::delete_tree(&targets.uninstall_key) {
        return Err("Não foi possível remover a entrada de Aplicativos instalados.".to_string());
    }
    if !settings::erase(&targets.settings_key) {
        return Err("Não foi possível apagar as preferências salvas.".to_string());
    }
    remove_directory(&targets.install_dir, running_exe)?;
    Ok(Removal::Removed)
}

fn message(
    text: &str,
    style: MESSAGEBOX_STYLE,
) -> windows::Win32::UI::WindowsAndMessaging::MESSAGEBOX_RESULT {
    unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(text),
            &HSTRING::from(PRODUCT),
            style | MB_SETFOREGROUND,
        )
    }
}

fn open_installed(exe: &Path) {
    let directory = exe
        .parent()
        .map(|parent| HSTRING::from(parent.as_os_str()))
        .unwrap_or_default();
    unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            &HSTRING::from(exe.as_os_str()),
            PCWSTR::null(),
            &directory,
            SW_SHOWNORMAL,
        );
    }
}

pub fn run_install(silent: bool) -> i32 {
    let result = Targets::for_user().and_then(|targets| {
        let source = std::env::current_exe().map_err(|error| {
            format!("Não foi possível localizar o próprio aplicativo.\n\n{error}")
        })?;
        install(&source, &targets).map(|()| targets)
    });
    match result {
        Ok(targets) => {
            if !silent {
                message(
                    "Sistemas Lineares foi instalado.\n\nAbra pelo menu Iniciar. Para remover, use Configurações > Aplicativos > Aplicativos instalados.",
                    MB_OK | MB_ICONINFORMATION,
                );
                open_installed(&targets.exe());
            }
            EXIT_OK
        }
        Err(error) => {
            if !silent {
                message(
                    &format!("A instalação não foi concluída.\n\n{error}"),
                    MB_OK | MB_ICONERROR,
                );
            }
            EXIT_ERROR
        }
    }
}

pub fn run_uninstall(silent: bool) -> i32 {
    let targets = match Targets::for_user() {
        Ok(targets) => targets,
        Err(error) => {
            if !silent {
                message(&error, MB_OK | MB_ICONERROR);
            }
            return EXIT_ERROR;
        }
    };
    if !silent {
        let answer = message(
            "Remover o Sistemas Lineares deste computador?\n\nO atalho do menu Iniciar, a entrada em Aplicativos instalados e as preferências salvas serão apagados.",
            MB_YESNO | MB_ICONQUESTION,
        );
        if answer != IDYES {
            return EXIT_CANCELLED;
        }
    }
    let running = std::env::current_exe().unwrap_or_default();
    match uninstall(&targets, &running) {
        Ok(removal) => {
            if !silent {
                let text = match removal {
                    Removal::Removed => "Sistemas Lineares foi removido deste computador.",
                    Removal::NothingInstalled => {
                        "O Sistemas Lineares não está instalado neste computador."
                    }
                };
                message(text, MB_OK | MB_ICONINFORMATION);
            }
            EXIT_OK
        }
        Err(error) => {
            if !silent {
                message(
                    &format!("A desinstalação não foi concluída.\n\n{error}"),
                    MB_OK | MB_ICONERROR,
                );
            }
            EXIT_ERROR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreta_argumentos() {
        let parse = |values: &[&str]| parse_arguments(values.iter().map(|value| value.to_string()));
        assert_eq!(
            parse(&[]),
            Arguments {
                command: None,
                silent: false
            }
        );
        assert_eq!(
            parse(&["--instalar"]),
            Arguments {
                command: Some(Command::Install),
                silent: false
            }
        );
        assert_eq!(
            parse(&["--DESINSTALAR", "--silencioso"]),
            Arguments {
                command: Some(Command::Uninstall),
                silent: true
            }
        );
        assert_eq!(
            parse(&["--desinstalar", "--instalar"]),
            Arguments {
                command: Some(Command::Uninstall),
                silent: false
            }
        );
        assert_eq!(
            parse(&["--qualquer", "x", "/?"]),
            Arguments {
                command: None,
                silent: false
            }
        );
    }

    #[test]
    fn comando_de_remocao_sai_assim_que_apaga() {
        let pasta = removal_command(
            Path::new(r"C:\Users\Fulano\AppData\Local\Programs\Sistemas Lineares"),
            true,
        );
        assert!(pasta.starts_with("cmd.exe /d /q /s /c \"for /l %n in (1,1,600) do (if exist \"C:\\Users\\Fulano\\AppData\\Local\\Programs\\Sistemas Lineares\" (rd /s /q \"C:\\Users\\Fulano\\AppData\\Local\\Programs\\Sistemas Lineares\" >nul 2>&1 & if exist "));
        assert!(
            pasta
                .to_lowercase()
                .contains("ping.exe\" -n 2 127.0.0.1 >nul) else exit) else exit)\"")
        );
        let arquivo = removal_command(
            Path::new(r"C:\Temp\SistemasLineares-removido-1-2.exe"),
            false,
        );
        assert!(arquivo.contains(
            "(del /f /q \"C:\\Temp\\SistemasLineares-removido-1-2.exe\" >nul 2>&1 & if exist "
        ));
        assert!(!arquivo.contains("rd /s /q"));
    }
}
