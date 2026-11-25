use std::ffi::OsStr;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use sistemas_lineares_ui::theme::{Color, Palette, SystemColors};

pub const ENVIRONMENT: &str = "SISTEMAS_LINEARES_ALTO_CONTRASTE";
const PROBE_TIMEOUT: Duration = Duration::from_millis(400);

pub const SYSTEM: SystemColors = SystemColors {
    window: Color::rgb(0, 0, 0),
    window_text: Color::rgb(255, 255, 255),
    button_face: Color::rgb(0, 0, 0),
    button_text: Color::rgb(255, 255, 255),
    highlight: Color::rgb(26, 235, 255),
    highlight_text: Color::rgb(0, 0, 0),
    gray_text: Color::rgb(63, 242, 63),
    hot_light: Color::rgb(255, 255, 0),
};

pub fn palette(high_contrast: bool) -> Palette {
    if high_contrast {
        Palette::high_contrast(SYSTEM)
    } else {
        Palette::dark()
    }
}

pub fn from_environment(value: Option<&OsStr>) -> Option<bool> {
    let text = value?.to_str()?.trim().to_lowercase();
    match text.as_str() {
        "1" | "sim" | "true" | "on" => Some(true),
        "0" | "nao" | "não" | "false" | "off" => Some(false),
        _ => None,
    }
}

pub fn requested(flag: bool, environment: Option<&OsStr>) -> Option<bool> {
    if flag {
        return Some(true);
    }
    from_environment(environment)
}

pub fn detect(flag: bool) -> bool {
    let environment = std::env::var_os(ENVIRONMENT);
    requested(flag, environment.as_deref()).unwrap_or_else(desktop_prefers_contrast)
}

pub(crate) fn run(program: &str, arguments: &[&str]) -> Option<String> {
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + PROBE_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                let mut output = String::new();
                let mut stdout = child.stdout.take()?;
                std::io::Read::read_to_string(&mut stdout, &mut output).ok()?;
                return Some(output);
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    }
}

pub fn portal_says_high(output: &str) -> bool {
    output.contains("uint32 1")
}

pub fn gsettings_says_high(output: &str) -> bool {
    output.trim() == "true"
}

fn desktop_prefers_contrast() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }
    let portal = run(
        "gdbus",
        &[
            "call",
            "--session",
            "--timeout",
            "1",
            "--dest",
            "org.freedesktop.portal.Desktop",
            "--object-path",
            "/org/freedesktop/portal/desktop",
            "--method",
            "org.freedesktop.portal.Settings.Read",
            "org.freedesktop.appearance",
            "contrast",
        ],
    );
    if let Some(output) = portal {
        return portal_says_high(&output);
    }
    run(
        "gsettings",
        &["get", "org.gnome.desktop.a11y.interface", "high-contrast"],
    )
    .is_some_and(|output| gsettings_says_high(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paleta_fixa_de_alto_contraste() {
        let paleta = palette(true);
        assert!(paleta.high_contrast);
        assert_eq!(paleta.canvas, Color::rgb(0, 0, 0));
        assert_eq!(paleta.ink, Color::rgb(255, 255, 255));
        assert_eq!(paleta.accent_fill, Color::rgb(26, 235, 255));
        assert_eq!(paleta.accent_text, Color::rgb(0, 0, 0));
        assert_eq!(paleta.disabled_text, Color::rgb(63, 242, 63));
        assert_eq!(paleta.link, Color::rgb(255, 255, 0));
        assert!(paleta.focus_glow.is_none());
        assert_eq!(palette(false), Palette::dark());
    }

    #[test]
    fn opcao_e_variavel_de_ambiente() {
        assert_eq!(requested(true, None), Some(true));
        assert_eq!(requested(true, Some(OsStr::new("0"))), Some(true));
        assert_eq!(requested(false, Some(OsStr::new("1"))), Some(true));
        assert_eq!(requested(false, Some(OsStr::new(" SIM "))), Some(true));
        assert_eq!(requested(false, Some(OsStr::new("0"))), Some(false));
        assert_eq!(requested(false, Some(OsStr::new("talvez"))), None);
        assert_eq!(requested(false, None), None);
    }

    #[test]
    fn interpreta_portal_e_gsettings() {
        assert!(portal_says_high("(<<uint32 1>>,)\n"));
        assert!(!portal_says_high("(<<uint32 0>>,)\n"));
        assert!(gsettings_says_high("true\n"));
        assert!(!gsettings_says_high("false\n"));
    }
}
