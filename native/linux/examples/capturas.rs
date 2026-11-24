use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use sistemas_lineares_core::Method;
use sistemas_lineares_linux::app::{self, App, Launch, Target};
use sistemas_lineares_linux::eframe::{self, egui};

const WIDTH: f32 = 1160.0;
const HEIGHT: f32 = 810.0;

#[derive(Clone)]
enum Step {
    Press(Target),
    Write(usize, usize, &'static str),
    Fill(&'static [&'static [&'static str]]),
    Events(Vec<egui::Event>),
    Hover(Target),
    Leave,
    Snap(&'static str),
    Full(&'static str),
    Size(f32, f32),
    Idle,
}

type Slot = Arc<Mutex<Option<Arc<egui::ColorImage>>>>;

enum Wait {
    Frames(u32),
    Input,
    Size(egui::Vec2),
    Time(f64),
    Screenshot(String, Slot),
}

struct Script {
    app: App,
    steps: VecDeque<Step>,
    directory: PathBuf,
    wait: Option<Wait>,
    inject: Vec<egui::Event>,
    pointer: Option<egui::Pos2>,
    restore: Option<egui::Vec2>,
    targets: Vec<(Target, egui::Rect)>,
}

fn key(key: egui::Key) -> Vec<egui::Event> {
    vec![
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        },
    ]
}

fn typing(text: &str) -> Vec<egui::Event> {
    vec![egui::Event::Text(text.to_string())]
}

fn tabs(count: usize) -> Vec<egui::Event> {
    (0..count).flat_map(|_| key(egui::Key::Tab)).collect()
}

fn save_png(path: &Path, image: &egui::ColorImage) -> Result<(), String> {
    let [width, height] = image.size;
    let bytes: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|pixel| {
            let [r, g, b, _] = pixel.to_array();
            [r, g, b, 255]
        })
        .collect();
    let buffer = image::RgbaImage::from_raw(width as u32, height as u32, bytes)
        .ok_or_else(|| "imagem inválida".to_string())?;
    buffer
        .save(path)
        .map_err(|error| format!("{}: {error}", path.display()))
}

impl Script {
    fn advance(&mut self, ctx: &egui::Context) {
        match self.wait.take() {
            Some(Wait::Frames(0)) | None => {}
            Some(Wait::Frames(count)) => {
                self.wait = Some(Wait::Frames(count - 1));
                return;
            }
            Some(Wait::Input) => {
                self.wait = Some(if self.inject.is_empty() && !self.app.input_pending() {
                    Wait::Frames(3)
                } else {
                    Wait::Input
                });
                return;
            }
            Some(Wait::Size(size)) => {
                let current = ctx.content_rect().size();
                if (current - size).length() > 0.5 {
                    self.wait = Some(Wait::Size(size));
                    return;
                }
                self.wait = Some(Wait::Frames(4));
                return;
            }
            Some(Wait::Time(until)) => {
                if ctx.input(|input| input.time) < until {
                    self.wait = Some(Wait::Time(until));
                    return;
                }
                self.wait = Some(Wait::Frames(3));
                return;
            }
            Some(Wait::Screenshot(name, slot)) => {
                let image = slot.lock().ok().and_then(|mut guard| guard.take());
                match image {
                    Some(image) => {
                        let path = self.directory.join(format!("{name}.png"));
                        match save_png(&path, &image) {
                            Ok(()) => println!(
                                "{name}  {}x{}  foco={:?} teclado={} janela={}",
                                image.size[0],
                                image.size[1],
                                self.app.focused(),
                                self.app.focus_visible(),
                                ctx.input(|input| input.focused)
                            ),
                            Err(error) => eprintln!("falha ao gravar {name}: {error}"),
                        }
                        if let Some(size) = self.restore.take() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                            self.wait = Some(Wait::Size(size));
                            return;
                        }
                    }
                    None => {
                        self.wait = Some(Wait::Screenshot(name, slot));
                        return;
                    }
                }
            }
        }
        let Some(step) = self.steps.pop_front() else {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        };
        match step {
            Step::Press(target) => {
                self.app.press(ctx, target);
                self.wait = Some(Wait::Frames(3));
            }
            Step::Write(row, column, text) => {
                self.app.write(ctx, row, column, text);
                self.wait = Some(Wait::Frames(3));
            }
            Step::Fill(rows) => {
                let dimension = rows.len();
                for (row, values) in rows.iter().enumerate() {
                    for (column, value) in values.iter().enumerate() {
                        let target = if column == dimension { 3 } else { column };
                        self.app.write(ctx, row, target, value);
                    }
                }
                self.wait = Some(Wait::Frames(3));
            }
            Step::Events(events) => {
                self.inject = events;
                self.wait = Some(Wait::Input);
            }
            Step::Hover(target) => {
                if let Some((_, rect)) = self.targets.iter().find(|(known, _)| *known == target) {
                    self.pointer = Some(rect.center());
                    self.inject.push(egui::Event::PointerMoved(rect.center()));
                }
                self.wait = Some(Wait::Frames(3));
            }
            Step::Leave => {
                self.pointer = None;
                self.inject.push(egui::Event::PointerGone);
                self.wait = Some(Wait::Frames(2));
            }
            Step::Snap(name) => self.snap(ctx, name),
            Step::Full(name) => {
                let height = self.app.content_size().1 as f32;
                let size = egui::vec2(WIDTH, height.max(HEIGHT));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                self.restore = Some(egui::vec2(WIDTH, HEIGHT));
                self.steps.push_front(Step::Snap(name));
                self.wait = Some(Wait::Size(size));
            }
            Step::Size(width, height) => {
                let size = egui::vec2(width, height);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                self.wait = Some(Wait::Size(size));
            }
            Step::Idle => {
                let now = ctx.input(|input| input.time);
                self.wait = Some(Wait::Time(now + app::IDLE_DELAY + 0.15));
            }
        }
    }

    fn snap(&mut self, ctx: &egui::Context, name: &str) {
        let slot: Slot = Arc::new(Mutex::new(None));
        let writer = Arc::clone(&slot);
        ctx.request_screenshot(move |image| {
            if let Ok(mut guard) = writer.lock() {
                *guard = Some(image);
            }
        });
        self.wait = Some(Wait::Screenshot(name.to_string(), slot));
    }

    fn remember_targets(&mut self, ctx: &egui::Context) {
        self.targets = Target::all()
            .into_iter()
            .filter_map(|target| {
                ctx.read_response(target.id())
                    .map(|response| (target, response.rect))
            })
            .collect();
    }
}

impl eframe::App for Script {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        raw_input.events.append(&mut self.inject);
        eframe::App::raw_input_hook(&mut self.app, ctx, raw_input);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        eframe::App::ui(&mut self.app, ui, frame);
        let ctx = ui.ctx().clone();
        self.remember_targets(&ctx);
        self.advance(&ctx);
        ctx.request_repaint();
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        eframe::App::clear_color(&self.app, visuals)
    }
}

fn script() -> Vec<Step> {
    use Step::*;
    let mut steps = vec![
        Snap("01-inicial-2x2"),
        Press(Target::Example),
        Snap("02-exemplo-2x2-pronto"),
        Press(Target::Solve),
        Snap("03-exemplo-2x2-resolvido"),
        Press(Target::Think),
        Snap("04-modo-pensar-2x2"),
        Full("05-modo-pensar-gauss-jordan-2x2"),
        Press(Target::Tab(Method::Cramer)),
        Full("06-modo-pensar-cramer-2x2"),
        Press(Target::Tab(Method::Gauss)),
        Full("07-modo-pensar-escalonamento-2x2"),
        Press(Target::Back),
        Press(Target::Three),
        Press(Target::Example),
        Press(Target::Solve),
        Snap("08-exemplo-3x3-resolvido"),
        Press(Target::Think),
        Press(Target::Tab(Method::Cramer)),
        Full("09-modo-pensar-cramer-3x3"),
        Press(Target::Tab(Method::GaussJordan)),
        Full("10-modo-pensar-gauss-jordan-3x3"),
        Press(Target::Tab(Method::Gauss)),
        Full("11-modo-pensar-escalonamento-3x3"),
        Press(Target::Back),
        Fill(&[
            &["1", "2", "3", "4"],
            &["2", "4", "6", "8"],
            &["3", "6", "9", "12"],
        ]),
        Press(Target::Solve),
        Snap("12-infinitas-3x3"),
        Press(Target::Think),
        Press(Target::Tab(Method::Cramer)),
        Full("13-modo-pensar-cramer-singular"),
        Press(Target::Back),
        Write(2, 3, "13"),
        Press(Target::Solve),
        Snap("14-sem-solucao-3x3"),
        Press(Target::Think),
        Press(Target::Tab(Method::Gauss)),
        Full("15-modo-pensar-sem-solucao"),
        Press(Target::Back),
        Press(Target::Clear),
        Press(Target::Two),
        Press(Target::Solve),
        Snap("16-erro-campo-vazio-2x2"),
        Fill(&[&["3", "0", "1"], &["0", "7", "1e-30"]]),
        Press(Target::Solve),
        Snap("17-valores-longos-2x2"),
        Press(Target::Think),
        Full("18-modo-pensar-valores-longos"),
        Press(Target::Back),
        Write(0, 0, "1/0"),
        Press(Target::Solve),
        Snap("19-erro-denominador-zero"),
        Fill(&[&["1", "1", "2"], &["2", "2", "4"]]),
        Press(Target::Solve),
        Snap("20-infinitas-2x2"),
        Size(640.0, 480.0),
        Press(Target::Three),
        Snap("21-janela-minima-3x3"),
        Press(Target::Example),
        Press(Target::Solve),
        Press(Target::Think),
        Snap("22-janela-minima-modo-pensar"),
        Press(Target::Back),
        Size(WIDTH, HEIGHT),
        Press(Target::Two),
        Press(Target::Clear),
    ];
    let mut digitar = vec![];
    for (texto, avancar) in [
        ("2", true),
        ("1", true),
        ("5", true),
        ("1", true),
        ("-1", false),
    ] {
        digitar.extend(typing(texto));
        if avancar {
            digitar.extend(key(egui::Key::Tab));
        }
    }
    steps.extend([
        Events(digitar),
        Snap("23-parcial-com-previa"),
        Events(key(egui::Key::Tab)),
        Events(typing("1")),
        Snap("24-resolvido-ao-vivo"),
        Press(Target::Field(1, 3)),
        Events(typing("-")),
        Snap("25-esmaecido-enquanto-digita"),
        Events(typing("3")),
        Snap("26-novo-resultado-ao-vivo"),
        Press(Target::Field(0, 0)),
        Events(typing("2x")),
        Idle,
        Snap("27-erro-revelado-na-pausa"),
        Events(key(egui::Key::Enter)),
        Events(typing("2")),
        Events(key(egui::Key::Enter)),
        Snap("28-enter-resolve-com-tudo-preenchido"),
        Press(Target::Field(1, 3)),
        Events(tabs(1)),
        Snap("29-foco-de-teclado-no-resolver"),
        Events(tabs(5)),
        Snap("30-foco-no-link-do-rodape"),
        Press(Target::Clear),
        Hover(Target::Field(0, 1)),
        Snap("31-hover-no-campo"),
        Leave,
        Fill(&[&["1e999", "1e998", "1e997"], &["1", "-1", "1e996"]]),
        Idle,
        Snap("32-numeros-grandes"),
        Press(Target::Clear),
        Press(Target::Link(Method::Cramer)),
        Snap("33-link-sem-sistema"),
        Press(Target::Example),
        Press(Target::Link(Method::Cramer)),
        Events(tabs(1)),
        Snap("34-modo-pensar-foco-na-aba"),
        Events(key(egui::Key::Escape)),
        Snap("35-esc-devolve-o-foco-ao-link"),
        Press(Target::Field(0, 0)),
        Events(typing("2^3")),
        Snap("36-dica-le-o-valor-da-potencia"),
        Press(Target::Clear),
        Hover(Target::Think),
        Idle,
        Snap("37-dica-do-botao-desabilitado"),
        Leave,
        Press(Target::Three),
        Snap("38-tres-por-tres-vazio-com-grade-propria"),
    ]);
    steps
}

fn main() -> Result<(), String> {
    let directory = std::env::args()
        .skip(1)
        .find(|argument| !argument.starts_with("--"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("capturas"));
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let high_contrast = std::env::args().any(|argument| argument == "--alto-contraste");
    let preferences = std::env::temp_dir().join(format!(
        "sistemas-lineares-capturas-{}.ron",
        std::process::id()
    ));
    let mut options = app::native_options();
    options.centered = false;
    options.persist_window = false;
    options.persistence_path = Some(preferences.clone());
    options.viewport = options
        .viewport
        .with_position([0.0, 0.0])
        .with_inner_size([WIDTH, HEIGHT]);
    let launch = Launch {
        high_contrast,
        open_url: |_| true,
    };
    let steps: VecDeque<Step> = script().into();
    let result = eframe::run_native(
        "sistemas-lineares-capturas",
        options,
        Box::new(move |context| {
            let app = App::new(context, launch);
            Ok(Box::new(Script {
                app,
                steps,
                directory,
                wait: Some(Wait::Frames(5)),
                inject: Vec::new(),
                pointer: None,
                restore: None,
                targets: Vec::new(),
            }))
        }),
    )
    .map_err(|error| error.to_string());
    let _ = std::fs::remove_file(&preferences);
    result
}
