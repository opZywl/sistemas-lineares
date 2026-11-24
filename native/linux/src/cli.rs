use crate::install;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Run,
    Install,
    Uninstall,
    Version,
    Help,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arguments {
    pub command: Command,
    pub silent: bool,
    pub high_contrast: bool,
    pub unknown: Vec<String>,
}

pub fn parse<I: IntoIterator<Item = String>>(arguments: I) -> Arguments {
    let mut parsed = Arguments {
        command: Command::Run,
        silent: false,
        high_contrast: false,
        unknown: Vec::new(),
    };
    for argument in arguments {
        let key = argument.trim().to_lowercase();
        let command = match key.as_str() {
            "--instalar" => Some(Command::Install),
            "--desinstalar" => Some(Command::Uninstall),
            "--versao" | "--versão" | "-v" | "--version" => Some(Command::Version),
            "--ajuda" | "-h" | "--help" => Some(Command::Help),
            "--silencioso" => {
                parsed.silent = true;
                None
            }
            "--alto-contraste" => {
                parsed.high_contrast = true;
                None
            }
            _ => {
                parsed.unknown.push(argument);
                None
            }
        };
        if let Some(command) = command {
            if parsed.command == Command::Run {
                parsed.command = command;
            }
        }
    }
    parsed
}

pub fn version_text() -> String {
    format!("{} {}", install::PRODUCT, install::VERSION)
}

pub fn help_text() -> String {
    format!(
        "{} {}\n\
         {}.\n\
         \n\
         Uso: {} [opções]\n\
         \n\
         Sem opções, abre o aplicativo.\n\
         \n\
         Opções:\n\
         \x20 --instalar        Instala para o usuário atual: o comando em ~/.local/bin,\n\
         \x20                   o atalho no menu de aplicativos e os ícones.\n\
         \x20 --desinstalar     Remove o comando, o atalho, os ícones e as preferências salvas.\n\
         \x20 --silencioso      Com --instalar ou --desinstalar: não pergunta nem mostra mensagens.\n\
         \x20 --alto-contraste  Abre com as cores de alto contraste\n\
         \x20                   (o mesmo que {}=1).\n\
         \x20 --versao          Mostra a versão e sai.\n\
         \x20 --ajuda           Mostra esta ajuda e sai.\n\
         \n\
         Códigos de saída: 0 concluído, 1 erro, 2 desinstalação cancelada.\n",
        install::PRODUCT,
        install::VERSION,
        install::DESCRIPTION,
        install::APP_ID,
        crate::contrast::ENVIRONMENT,
    )
}

pub fn unknown_text(unknown: &[String]) -> Option<String> {
    if unknown.is_empty() {
        return None;
    }
    Some(format!(
        "Opção desconhecida: {}. Use {} --ajuda para ver as opções.",
        unknown.join(" "),
        install::APP_ID
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ler(valores: &[&str]) -> Arguments {
        parse(valores.iter().map(|valor| (*valor).to_string()))
    }

    #[test]
    fn interpreta_argumentos() {
        assert_eq!(ler(&[]).command, Command::Run);
        assert_eq!(ler(&["--instalar"]).command, Command::Install);
        let desinstalar = ler(&["--DESINSTALAR", "--silencioso"]);
        assert_eq!(desinstalar.command, Command::Uninstall);
        assert!(desinstalar.silent);
        assert_eq!(
            ler(&["--desinstalar", "--instalar"]).command,
            Command::Uninstall,
            "o primeiro comando vale"
        );
        assert!(ler(&["--alto-contraste"]).high_contrast);
        assert_eq!(ler(&["--versao"]).command, Command::Version);
        assert_eq!(ler(&["--versão"]).command, Command::Version);
        assert_eq!(ler(&["--ajuda"]).command, Command::Help);
        assert_eq!(ler(&["-h"]).command, Command::Help);
        let estranho = ler(&["--qualquer", "x"]);
        assert_eq!(estranho.command, Command::Run);
        assert_eq!(
            estranho.unknown,
            vec!["--qualquer".to_string(), "x".to_string()]
        );
    }

    #[test]
    fn textos_da_linha_de_comando() {
        assert_eq!(
            version_text(),
            format!("Sistemas Lineares {}", env!("CARGO_PKG_VERSION"))
        );
        let ajuda = help_text();
        for trecho in [
            "Uso: sistemas-lineares [opções]",
            "--instalar",
            "--desinstalar",
            "--silencioso",
            "--alto-contraste",
            "--versao",
            "--ajuda",
            "SISTEMAS_LINEARES_ALTO_CONTRASTE=1",
            "2 desinstalação cancelada",
        ] {
            assert!(ajuda.contains(trecho), "{trecho}");
        }
        assert_eq!(unknown_text(&[]), None);
        assert!(
            unknown_text(&["--x".to_string()])
                .unwrap_or_default()
                .contains("--x")
        );
    }
}
