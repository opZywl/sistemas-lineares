use std::ffi::OsStr;

use crate::contrast;

pub const ENVIRONMENT: &str = "SISTEMAS_LINEARES_ANIMACOES";

pub fn requested(environment: Option<&OsStr>) -> Option<bool> {
    contrast::from_environment(environment)
}

pub fn detect() -> bool {
    let environment = std::env::var_os(ENVIRONMENT);
    requested(environment.as_deref()).unwrap_or_else(desktop_allows_animations)
}

pub fn portal_says_reduced(output: &str) -> bool {
    output.contains("uint32 1")
}

pub fn gsettings_says_enabled(output: &str) -> bool {
    output.trim() != "false"
}

fn desktop_allows_animations() -> bool {
    if !cfg!(target_os = "linux") {
        return true;
    }
    let portal = contrast::run(
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
            "reduced-motion",
        ],
    );
    if let Some(output) = portal {
        return !portal_says_reduced(&output);
    }
    contrast::run(
        "gsettings",
        &["get", "org.gnome.desktop.interface", "enable-animations"],
    )
    .is_none_or(|output| gsettings_says_enabled(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variavel_de_ambiente_liga_e_desliga() {
        assert_eq!(requested(Some(OsStr::new("0"))), Some(false));
        assert_eq!(requested(Some(OsStr::new(" não "))), Some(false));
        assert_eq!(requested(Some(OsStr::new("off"))), Some(false));
        assert_eq!(requested(Some(OsStr::new("1"))), Some(true));
        assert_eq!(requested(Some(OsStr::new("SIM"))), Some(true));
        assert_eq!(requested(Some(OsStr::new("talvez"))), None);
        assert_eq!(requested(None), None);
    }

    #[test]
    fn interpreta_portal_e_gsettings() {
        assert!(portal_says_reduced("(<<uint32 1>>,)\n"));
        assert!(!portal_says_reduced("(<<uint32 0>>,)\n"));
        assert!(gsettings_says_enabled("true\n"));
        assert!(!gsettings_says_enabled("false\n"));
        assert!(
            gsettings_says_enabled(""),
            "sem resposta clara, as animações ficam ligadas"
        );
    }
}
