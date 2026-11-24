#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    std::process::exit(sistemas_lineares::run());
}

#[cfg(not(windows))]
fn main() {
    eprintln!(
        "Este executável é o app do Windows. No Linux, use o pacote sistemas-lineares-linux."
    );
    std::process::exit(1);
}
