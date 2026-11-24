use sistemas_lineares_linux::cli::{self, Command};
use sistemas_lineares_linux::{app, contrast, install};

fn main() {
    let arguments = cli::parse(
        std::env::args_os()
            .skip(1)
            .map(|argument| argument.to_string_lossy().into_owned()),
    );
    if let Some(warning) = cli::unknown_text(&arguments.unknown) {
        eprintln!("{warning}");
    }
    let code = match arguments.command {
        Command::Help => {
            print!("{}", cli::help_text());
            install::EXIT_OK
        }
        Command::Version => {
            println!("{}", cli::version_text());
            install::EXIT_OK
        }
        Command::Install => install::run_install(
            arguments.silent,
            &mut std::io::stdout(),
            &mut std::io::stderr(),
        ),
        Command::Uninstall => install::run_uninstall(arguments.silent),
        Command::Run => {
            let launch = app::Launch {
                high_contrast: contrast::detect(arguments.high_contrast),
                ..app::Launch::default()
            };
            match app::run(launch) {
                Ok(()) => install::EXIT_OK,
                Err(error) => {
                    eprintln!("Não foi possível iniciar o aplicativo.\n\n{error}");
                    install::EXIT_ERROR
                }
            }
        }
    };
    std::process::exit(code);
}
