#![cfg(windows)]

mod comum;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use comum::*;
use sistemas_lineares::fonts::{self, TextMetrics};
use sistemas_lineares::gfx::{self, Align, Engine, PlacedLine, VAlign, dot_centers};
use sistemas_lineares::graph::{self, Anchor, Ink, Item, Motion, Pt, Shape, Sketch};
use sistemas_lineares::layout::{Rect, d, dpi_scale};
use sistemas_lineares::model::{self, Model};
use sistemas_lineares::plot;
use sistemas_lineares::text::wrap;
use sistemas_lineares::theme::{self, Color, Font, Palette};
use sistemas_lineares::window::system_colors;
use sistemas_lineares::{
    Button, Diagnostics, Handles, Launch, Renderer, diagnostics, force_animations,
    force_high_contrast, pump,
};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, ROLE_SYSTEM_GRAPHIC, UIA_ImageControlTypeId,
};
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN;
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetWindowRect, IsIconic, SB_BOTTOM, SW_SHOWMINNOACTIVE, SW_SHOWNOACTIVATE,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOZORDER, SendMessageW, SetWindowPos, ShowWindow, WM_VSCROLL,
    WS_CLIPCHILDREN, WS_EX_CONTROLPARENT, WS_OVERLAPPEDWINDOW,
};

const LEGENDA_2X2: &str = "As retas se cruzam em (2, 1): esse ponto é a solução do sistema.";
const LEGENDA_3X3: &str =
    "Os três planos se cruzam em (1, 2, 3): esse ponto é a solução do sistema.";
const GRANDES: [[&str; 4]; 3] = [
    ["1234567", "2", "3", "7654321"],
    ["4", "2345678", "6", "8765432"],
    ["7", "8", "3456789", "9876543"],
];

fn estado(h: &Handles) -> Diagnostics {
    diagnostics(h.main).expect("diagnóstico")
}

fn preencher_3x3(h: &Handles, linhas: &[[&str; 4]; 3]) {
    for (linha, valores) in linhas.iter().enumerate() {
        for (coluna, valor) in valores.iter().enumerate() {
            escrever(h, linha, coluna, valor);
        }
    }
    tecla(h, h.inputs[2][3], VK_RETURN);
}

fn redimensionar(h: &Handles, largura: i32, altura: i32) {
    let mut janela = RECT::default();
    let mut cliente = RECT::default();
    unsafe {
        let _ = GetWindowRect(h.main, &mut janela);
        let _ = GetClientRect(h.main, &mut cliente);
        let _ = SetWindowPos(
            h.main,
            None,
            0,
            0,
            largura + (janela.right - janela.left) - cliente.right,
            altura + (janela.bottom - janela.top) - cliente.bottom,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
    pump(h.main);
}

fn plano_da_legenda(h: &Handles, atual: &Diagnostics) -> (Vec<PlacedLine>, TextMetrics, i32) {
    let motor = Engine::new(atual.dpi).expect("motor de desenho");
    let metrica = motor.text_metrics(Font::Small);
    let justa = fonts::preferred_height(Font::Small.pixels(atual.dpi)).expect("espaçamento");
    let legenda = atual.layout.helper;
    let medir = |linha: &str| motor.measure(linha, Font::Small);
    let linhas = gfx::plan_text(
        &sistemas_lineares::text::keep_groups(&texto(h.helper)),
        Rect::new(0, 0, legenda.w, legenda.h),
        Align::Left,
        VAlign::Center,
        metrica,
        justa,
        &medir,
    );
    (linhas, metrica, justa)
}

fn linhas_com_tinta(imagem: &Imagem, regiao: Rect, cor: Color) -> Vec<i32> {
    (regiao.y..regiao.bottom())
        .filter(|y| {
            imagem.conta((regiao.x, *y, regiao.right(), *y + 1), |pixel| {
                perto(pixel, cor, 60)
            }) > 0
        })
        .collect()
}

fn no_anel(imagem: &Imagem, centro: (f64, f64), raios: (f64, f64), cor: Color) -> usize {
    let (dentro, fora) = raios;
    let mut total = 0;
    let alcance = fora.ceil() as i32 + 1;
    let (cx, cy) = (centro.0.floor() as i32, centro.1.floor() as i32);
    for y in cy - alcance..=cy + alcance {
        for x in cx - alcance..=cx + alcance {
            let dx = f64::from(x) + 0.5 - centro.0;
            let dy = f64::from(y) + 0.5 - centro.1;
            let distancia_ao_centro = (dx * dx + dy * dy).sqrt();
            if (dentro..=fora).contains(&distancia_ao_centro) && perto(imagem.cor(x, y), cor, 40) {
                total += 1;
            }
        }
    }
    total
}

fn automacao(h: &Handles, alvo: HWND) -> Option<(i32, String, String)> {
    let alvo = alvo.0 as isize;
    let concluido = Arc::new(AtomicBool::new(false));
    let sinal = concluido.clone();
    let (envio, recebe) = mpsc::channel();
    let trabalhador = std::thread::spawn(move || {
        let resultado = unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let servico: windows::core::Result<IUIAutomation> =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER);
            servico.ok().and_then(|servico| {
                let elemento = servico.ElementFromHandle(HWND(alvo as *mut _)).ok()?;
                let tipo = elemento.CurrentControlType().ok()?;
                let nome = elemento.CurrentName().ok()?.to_string();
                let ajuda = elemento
                    .CurrentHelpText()
                    .map(|texto| texto.to_string())
                    .unwrap_or_default();
                Some((tipo.0, nome, ajuda))
            })
        };
        let _ = envio.send(resultado);
        sinal.store(true, Ordering::SeqCst);
    });
    let limite = Instant::now() + Duration::from_secs(15);
    while !concluido.load(Ordering::SeqCst) && Instant::now() < limite {
        pump(h.main);
        std::thread::sleep(Duration::from_millis(5));
    }
    let resultado = recebe.recv_timeout(Duration::from_secs(1)).ok().flatten();
    let _ = trabalhador.join();
    resultado
}

fn exemplo(dimensao: usize) -> Model {
    let mut modelo = Model::new();
    modelo.change_dimension(dimensao);
    let _ = modelo.fill_example();
    modelo
}

fn repouso(sketch: &Sketch, estado: &Diagnostics, contraste: bool) -> Vec<Item> {
    let plot = estado.layout.plot;
    let mut motion = Motion::new();
    motion.update(sketch, 0.0, false);
    motion.scene(
        1e9,
        f64::from(plot.w),
        f64::from(plot.h),
        f64::from(dpi_scale(estado.dpi)),
        contraste,
    )
}

fn centro_do_ponto(itens: &[Item], plot: Rect) -> (i32, i32) {
    let centro = itens
        .iter()
        .find_map(|item| match (&item.shape, item.ink) {
            (Shape::Disc { center, .. }, Ink::Point) => Some(*center),
            _ => None,
        })
        .expect("a cena tem o ponto da solução");
    (
        plot.x + centro.x.floor() as i32,
        plot.y + centro.y.floor() as i32,
    )
}

fn perto(cor: Color, alvo: Color, tolerancia: i32) -> bool {
    distancia(cor, alvo) < tolerancia
}

fn numero_colorido(imagem: &Imagem, regiao: Rect, cor: Color) -> usize {
    imagem.conta(area(regiao), |pixel| {
        let ate_a_cor = distancia(pixel, cor);
        ate_a_cor < 90 && ate_a_cor < distancia(pixel, theme::MUTED)
    })
}

#[test]
fn exemplo_2x2_desenha_retas_ponto_e_textura() {
    for (nome, renderer) in [("Direct2D", Renderer::Direct2D), ("GDI", Renderer::Gdi)] {
        let h = janela_com(Launch {
            renderer,
            animations: Some(false),
            ..Launch::background(FORA_DA_TELA, CLIENTE)
        });
        clicar(&h, Button::Example);
        let atual = estado(&h);
        assert!(
            !atual.graph_timer && !atual.graph_busy,
            "{nome}: em repouso"
        );
        let plot = atual.layout.plot;
        let painel = atual.layout.graph;
        assert!(
            painel.contains(plot) && !plot.is_empty(),
            "{nome}: {plot:?}"
        );
        assert!(visivel(h.graph), "{nome}: gráfico visível");
        assert_eq!(
            posicao_no_canvas(&h, h.graph),
            plot,
            "{nome}: gráfico em layout.plot"
        );
        assert_eq!(
            posicao_no_canvas(&h, h.helper),
            atual.layout.helper,
            "{nome}: legenda em layout.helper"
        );
        let esperado = repouso(&exemplo(2).sketch(), &atual, false);
        assert_eq!(
            atual.graph_scene,
            graph::describe(&esperado),
            "{nome}: a cena desenhada é a do núcleo compartilhado"
        );
        assert_eq!(texto(h.helper), LEGENDA_2X2);
        assert_ne!(texto(h.helper), model::EXAMPLE_HELPER);

        let imagem = capturar(h.main);
        let regiao = area(plot);
        let lilas = imagem.conta(regiao, |cor| perto(cor, theme::ACCENT, 40));
        let menta = imagem.conta(regiao, |cor| perto(cor, theme::MINT, 40));
        eprintln!("{nome}: {lilas} pixels lilás, {menta} pixels menta");
        assert!(lilas > 80, "{nome}: reta da equação 1 em lilás ({lilas})");
        assert!(menta > 80, "{nome}: reta da equação 2 em menta ({menta})");
        let espaco = 8.0 * dpi_scale(atual.dpi);
        let mut pontinhos = 0;
        for y in dot_centers(plot.h, espaco) {
            for x in dot_centers(plot.w, espaco) {
                let cor = imagem.cor(plot.x + x as i32 - 1, plot.y + y as i32 - 1);
                if distancia(cor, theme::CANVAS) >= 24 {
                    pontinhos += 1;
                }
            }
        }
        eprintln!("{nome}: {pontinhos} pontinhos da textura");
        assert!(
            pontinhos > 300,
            "{nome}: textura de pontinhos ({pontinhos})"
        );
        let (x, y) = centro_do_ponto(&esperado, plot);
        assert!(
            perto(imagem.cor(x, y), theme::INK, 30),
            "{nome}: ponto da solução em ({x},{y}) {:?}",
            imagem.cor(x, y)
        );
        let fora = imagem.conta((plot.x - 3, plot.y - 3, plot.right() + 3, plot.y), |cor| {
            perto(cor, theme::ACCENT, 60) || perto(cor, theme::MINT, 60)
        });
        assert_eq!(fora, 0, "{nome}: nada vaza acima de layout.plot");
        let meio = painel.y + painel.h / 2;
        assert!(
            perto(imagem.cor(painel.x, meio), theme::BORDER, 16),
            "{nome}: contorno do painel {:?}",
            imagem.cor(painel.x, meio)
        );
        assert_eq!(
            imagem.cor(painel.right() - 6, meio),
            theme::CANVAS,
            "{nome}: fundo do painel"
        );
        let legenda = atual.layout.helper;
        assert_eq!(
            imagem.cor(legenda.x + 1, legenda.y + 1),
            theme::CANVAS,
            "{nome}: fundo da legenda igual ao do painel"
        );
        let tinta = imagem.conta(area(legenda), |cor| perto(cor, theme::MUTED, 60));
        assert!(tinta > 150, "{nome}: legenda desenhada ({tinta})");
        let numeros = atual.layout.row_numbers;
        let lilas = numero_colorido(&imagem, numeros[0], theme::ACCENT);
        let menta = numero_colorido(&imagem, numeros[1], theme::MINT);
        assert!(
            lilas > 5 && menta > 5,
            "{nome}: números das equações nas cores das retas ({lilas}, {menta})"
        );
        fechar(&h);
    }
}

#[test]
fn tres_por_tres_mostra_os_planos_e_a_solucao() {
    let h = janela_animada(false);
    clicar(&h, Button::Three);
    assert_eq!(texto(h.helper), graph::INTRO_THREE);
    clicar(&h, Button::Example);
    let atual = estado(&h);
    assert!(!atual.graph_timer);
    let plot = atual.layout.plot;
    let esperado = repouso(&exemplo(3).sketch(), &atual, false);
    assert_eq!(atual.graph_scene, graph::describe(&esperado));
    assert_eq!(texto(h.helper), LEGENDA_3X3);
    assert_eq!(posicao_no_canvas(&h, h.graph), plot);
    let planos = esperado
        .iter()
        .filter(|item| {
            matches!(item.ink, Ink::Row(_)) && matches!(item.shape, Shape::Polygon { .. })
        })
        .count();
    assert_eq!(planos, 3, "três planos translúcidos");
    let imagem = capturar(h.main);
    let regiao = area(plot);
    for (linha, cor) in [theme::ACCENT, theme::MINT, theme::PEACH]
        .into_iter()
        .enumerate()
    {
        let pixels = imagem.conta(regiao, |pixel| perto(pixel, cor, 60));
        assert!(pixels > 25, "plano da equação {} ({pixels})", linha + 1);
        let tinta = numero_colorido(&imagem, atual.layout.row_numbers[linha], cor);
        assert!(
            tinta > 5,
            "número da equação {} colorido ({tinta})",
            linha + 1
        );
    }
    let (x, y) = centro_do_ponto(&esperado, plot);
    assert!(perto(imagem.cor(x, y), theme::INK, 30), "ponto da solução");
    fechar(&h);
}

#[test]
fn legenda_e_descricao_acessivel_acompanham_o_sistema() {
    let h = janela_animada(false);
    assert_eq!(texto(h.graph), graph::GRAPH_NAME);
    assert_eq!(nome_acessivel(h.graph), graph::GRAPH_NAME);
    assert_eq!(
        papel_acessivel(h.graph),
        Some(ROLE_SYSTEM_GRAPHIC as i32),
        "o painel é uma imagem para o leitor de tela"
    );
    assert_eq!(texto(h.helper), graph::INTRO_TWO);
    assert_eq!(descricao_acessivel(h.graph), graph::INTRO_TWO);
    assert_eq!(nome_acessivel(h.helper), graph::INTRO_TWO);
    match automacao(&h, h.graph) {
        Some((tipo, nome, ajuda)) => {
            assert_eq!(
                tipo, UIA_ImageControlTypeId.0,
                "UI Automation vê o painel como imagem"
            );
            assert_eq!(nome, graph::GRAPH_NAME);
            assert_eq!(
                ajuda,
                graph::INTRO_TWO,
                "leitores via UI Automation também recebem a legenda"
            );
        }
        None => eprintln!(
            "aviso: UI Automation indisponível neste ambiente; papel conferido só pelo MSAA"
        ),
    }
    assert_eq!(
        estado(&h).live_regions,
        5,
        "a legenda não vira uma região viva nova"
    );
    escrever(&h, 0, 0, "2");
    escrever(&h, 0, 1, "1");
    escrever(&h, 0, 3, "5");
    let parcial = "Preencha a equação 2 para ver onde as retas se cruzam.";
    assert_eq!(texto(h.helper), parcial);
    assert_eq!(descricao_acessivel(h.graph), parcial);
    escrever(&h, 1, 0, "1");
    escrever(&h, 1, 1, "-1");
    escrever(&h, 1, 3, "1");
    assert_eq!(texto(h.helper), LEGENDA_2X2);
    assert_eq!(descricao_acessivel(h.graph), LEGENDA_2X2);
    escrever(&h, 1, 3, "5/2");
    escrever(&h, 1, 0, "2");
    escrever(&h, 1, 1, "1");
    assert_eq!(texto(h.helper), graph::PARALLEL_LINES);
    assert_eq!(descricao_acessivel(h.graph), graph::PARALLEL_LINES);
    escrever(&h, 1, 3, "5");
    assert_eq!(texto(h.helper), graph::COINCIDENT_LINES);
    escrever(&h, 0, 0, "1/0");
    ocioso(&h);
    assert!(
        texto(h.helper).starts_with("Equação 1, coeficiente de x:"),
        "o erro continua tendo prioridade na legenda: {}",
        texto(h.helper)
    );
    assert_eq!(
        descricao_acessivel(h.graph),
        "Preencha a equação 1 para ver onde as retas se cruzam.",
        "a descrição do gráfico continua descrevendo o desenho"
    );
    clicar(&h, Button::Clear);
    assert_eq!(texto(h.helper), graph::INTRO_TWO);
    assert_eq!(descricao_acessivel(h.graph), graph::INTRO_TWO);
    fechar(&h);
}

#[test]
fn animacao_termina_e_o_temporizador_para_sozinho() {
    let h = janela_animada(true);
    assert!(estado(&h).animations);
    assert!(!estado(&h).graph_timer, "parado no início");
    let inicio = Instant::now();
    clicar(&h, Button::Example);
    let comeco = estado(&h);
    assert!(
        comeco.graph_timer && comeco.graph_busy,
        "o exemplo anima as retas e o ponto"
    );
    assert_ne!(
        comeco.graph_scene,
        graph::describe(&repouso(&exemplo(2).sketch(), &comeco, false)),
        "o primeiro quadro ainda não é o repouso"
    );
    let quadros = comeco.graph_frames;
    assert!(esperar_grafico(&h), "a animação precisa terminar");
    let duracao = inicio.elapsed();
    let fim = estado(&h);
    assert!(!fim.graph_timer && !fim.graph_busy);
    eprintln!(
        "animação do exemplo: {duracao:?}, {} quadros",
        fim.graph_frames - quadros
    );
    assert!(
        duracao < Duration::from_secs(5),
        "a animação durou {duracao:?}"
    );
    assert!(
        fim.graph_frames >= quadros + 15,
        "quadros pintados durante a animação: {}",
        fim.graph_frames - quadros
    );
    assert_eq!(
        fim.graph_scene,
        graph::describe(&repouso(&exemplo(2).sketch(), &fim, false)),
        "termina exatamente na cena de repouso"
    );
    let parado = fim.graph_frames;
    let limite = Instant::now() + Duration::from_millis(400);
    while Instant::now() < limite {
        pump(h.main);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        estado(&h).graph_frames,
        parado,
        "parado, o gráfico não repinta sozinho"
    );
    escrever(&h, 0, 3, "6");
    assert!(estado(&h).graph_timer, "mudar um valor anima de novo");
    assert!(esperar_grafico(&h));
    escrever(&h, 0, 3, "6");
    assert!(!estado(&h).graph_timer, "o mesmo sistema não reanima");
    clicar(&h, Button::Three);
    assert!(
        !estado(&h).graph_timer,
        "trocar de tamanho troca o gráfico sem animação"
    );
    fechar(&h);
}

#[test]
fn sem_animacoes_nao_ha_quadros_extras() {
    let h = janela_animada(false);
    assert!(!estado(&h).animations);
    clicar(&h, Button::Example);
    let atual = estado(&h);
    assert!(!atual.graph_timer && !atual.graph_busy);
    assert_eq!(
        atual.graph_scene,
        graph::describe(&repouso(&exemplo(2).sketch(), &atual, false)),
        "sem animação, o gráfico já aparece pronto"
    );
    let quadros = atual.graph_frames;
    let limite = Instant::now() + Duration::from_millis(300);
    while Instant::now() < limite {
        pump(h.main);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(estado(&h).graph_frames, quadros, "nenhum laço de quadros");

    force_animations(h.main, Some(true));
    escrever(&h, 0, 0, "4");
    let animando = estado(&h);
    assert!(animando.animations && animando.graph_timer && animando.graph_busy);
    force_animations(h.main, Some(false));
    pump(h.main);
    let parado = estado(&h);
    assert!(
        !parado.graph_timer && !parado.graph_busy,
        "desligar as animações no meio leva tudo ao repouso"
    );
    let mut modelo = Model::new();
    let _ = modelo.fill_example();
    modelo.edit(0, 0, "4");
    assert_eq!(
        parado.graph_scene,
        graph::describe(&repouso(&modelo.sketch(), &parado, false))
    );
    fechar(&h);
}

#[test]
fn alto_contraste_sem_alfa_sem_textura_e_com_cores_do_sistema() {
    let h = janela_com(Launch {
        high_contrast: Some(true),
        animations: Some(false),
        ..Launch::background(FORA_DA_TELA, CLIENTE)
    });
    clicar(&h, Button::Example);
    let atual = estado(&h);
    assert!(atual.palette.high_contrast);
    let esperado = repouso(&exemplo(2).sketch(), &atual, true);
    assert_eq!(atual.graph_scene, graph::describe(&esperado));
    assert!(esperado.iter().all(|item| item.alpha == 1.0), "sem alfa");
    assert!(
        !esperado
            .iter()
            .any(|item| matches!(item.shape, Shape::Dots { .. })),
        "sem textura"
    );
    assert!(
        !esperado.iter().any(|item| item.ink == Ink::Halo),
        "sem brilho"
    );
    let sistema = system_colors();
    let paleta = Palette::high_contrast(sistema);
    let imagem = capturar(h.main);
    let plot = atual.layout.plot;
    let painel = atual.layout.graph;
    assert_eq!(
        imagem.cor(painel.right() - 6, painel.y + painel.h / 2),
        sistema.window,
        "fundo do painel com COLOR_WINDOW"
    );
    assert!(
        perto(
            imagem.cor(painel.x, painel.y + painel.h / 2),
            sistema.window_text,
            40
        ),
        "contorno do painel com COLOR_WINDOWTEXT"
    );
    let reta = imagem.conta(area(plot), |cor| perto(cor, paleta.graph_rows[0], 40));
    assert!(reta > 100, "reta com a cor do sistema ({reta})");
    let (x, y) = centro_do_ponto(&esperado, plot);
    assert!(
        perto(imagem.cor(x, y), paleta.graph_point, 30),
        "ponto com a cor do sistema"
    );
    force_high_contrast(h.main, Some(false));
    pump(h.main);
    let escuro = estado(&h);
    assert!(!escuro.palette.high_contrast);
    assert!(
        escuro.graph_scene.starts_with("t1000:D"),
        "fora do alto contraste a textura volta"
    );
    fechar(&h);
}

#[test]
fn sair_do_alto_contraste_durante_o_pulso_retoma_a_animacao() {
    let h = janela_com(Launch {
        high_contrast: Some(true),
        animations: Some(true),
        ..Launch::background(FORA_DA_TELA, CLIENTE)
    });
    let clique = Instant::now();
    clicar(&h, Button::Example);
    let comeco = estado(&h);
    assert!(
        comeco.palette.high_contrast && comeco.graph_timer && comeco.graph_busy,
        "com alto contraste, o exemplo anima as retas e o ponto"
    );
    assert!(esperar_grafico(&h), "a animação com alto contraste termina");
    let limite = clique + Duration::from_secs(1);
    while Instant::now() < limite {
        pump(h.main);
        std::thread::sleep(Duration::from_millis(10));
    }
    let parado = estado(&h);
    assert!(
        !parado.graph_timer && !parado.graph_busy,
        "com alto contraste, o ponto não pulsa"
    );
    force_high_contrast(h.main, Some(false));
    pump(h.main);
    let pulsando = estado(&h);
    assert!(!pulsando.palette.high_contrast);
    assert!(
        pulsando.graph_busy && pulsando.graph_timer,
        "sem alto contraste, os pulsos que faltam voltam a animar"
    );
    assert!(esperar_grafico(&h), "e os pulsos terminam sozinhos");
    let fim = estado(&h);
    assert!(!fim.graph_timer && !fim.graph_busy);
    assert_eq!(
        fim.graph_scene,
        graph::describe(&repouso(&exemplo(2).sketch(), &fim, false)),
        "termina na cena de repouso sem alto contraste"
    );
    fechar(&h);
}

fn rolar_ate_o_fim(h: &Handles) {
    unsafe {
        SendMessageW(
            h.main,
            WM_VSCROLL,
            Some(WPARAM(SB_BOTTOM.0 as usize)),
            Some(LPARAM(0)),
        );
    }
    pump(h.main);
}

fn cores_dos_planos(imagem: &Imagem, plot: Rect) -> [usize; 3] {
    let regiao = area(plot);
    [theme::ACCENT, theme::MINT, theme::PEACH]
        .map(|cor| imagem.conta(regiao, |pixel| perto(pixel, cor, 60)))
}

#[test]
fn janela_minima_tem_painel_tambem_no_3x3() {
    let h = janela_com(Launch {
        animations: Some(false),
        ..Launch::background(FORA_DA_TELA, (640, 480))
    });
    clicar(&h, Button::Three);
    let atual = estado(&h);
    let layout = atual.layout;
    let escala = dpi_scale(atual.dpi);
    assert!(
        !layout.graph.is_empty(),
        "na janela mínima, o 3×3 também tem painel"
    );
    assert!(
        layout.plot.h >= d(escala, 70),
        "plot com altura útil: {:?}",
        layout.plot
    );
    assert!(layout.graph.y > layout.fields[2][3].bottom());
    assert!(layout.graph.contains(layout.plot) && layout.graph.contains(layout.helper));
    assert!(visivel(h.graph) && visivel(h.helper));
    assert_eq!(posicao_no_canvas(&h, h.graph), layout.plot);
    assert_eq!(posicao_no_canvas(&h, h.helper), layout.helper);
    assert_eq!(texto(h.helper), graph::INTRO_THREE);
    assert!(
        !atual.graph_scene.is_empty(),
        "o painel vazio já desenha os eixos"
    );
    rolar_ate_o_fim(&h);
    let rolagem = estado(&h).scroll;
    let imagem = capturar(h.main);
    let legenda = layout.helper.offset(-rolagem.0, -rolagem.1);
    let painel = layout.graph.offset(-rolagem.0, -rolagem.1);
    assert!(
        painel.y >= 0 && painel.bottom() <= imagem.altura && painel.right() <= imagem.largura,
        "painel à vista depois de rolar: {painel:?} em {}x{}",
        imagem.largura,
        imagem.altura
    );
    let linhas = linhas_com_tinta(&imagem, legenda, theme::MUTED);
    let altura = linhas
        .last()
        .zip(linhas.first())
        .map_or(0, |(fim, inicio)| fim - inicio);
    assert!(
        altura > 18,
        "a legenda quebra em mais de uma linha sem cortar ({altura} px)"
    );
    assert!(
        linhas.first().is_some_and(|topo| *topo > legenda.y)
            && linhas
                .last()
                .is_some_and(|base| *base < legenda.bottom() - 1),
        "a legenda fica centralizada no painel"
    );
    assert_eq!(
        imagem.cor(legenda.x + 1, legenda.y + 1),
        theme::CANVAS,
        "a legenda usa o fundo do painel"
    );
    clicar(&h, Button::Example);
    let resolvido = estado(&h);
    let esperado = repouso(&exemplo(3).sketch(), &resolvido, false);
    assert_eq!(resolvido.graph_scene, graph::describe(&esperado));
    assert_eq!(texto(h.helper), LEGENDA_3X3);
    let rolagem = resolvido.scroll;
    let imagem = capturar(h.main);
    let plot = layout.plot.offset(-rolagem.0, -rolagem.1);
    for (linha, pixels) in cores_dos_planos(&imagem, plot).into_iter().enumerate() {
        assert!(
            pixels > 25,
            "plano da equação {} na janela mínima ({pixels})",
            linha + 1
        );
    }
    clicar(&h, Button::Two);
    assert!(
        !estado(&h).layout.graph.is_empty(),
        "no 2×2 o painel continua"
    );
    assert!(visivel(h.graph));
    assert_eq!(texto(h.helper), graph::INTRO_TWO);
    fechar(&h);
}

#[test]
fn janela_do_usuario_mostra_os_planos_no_3x3() {
    let externo = (1352, 798);
    let mut moldura = RECT::default();
    unsafe {
        let _ = AdjustWindowRectExForDpi(
            &mut moldura,
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            false,
            WS_EX_CONTROLPARENT,
            96,
        );
    }
    let cliente = (
        externo.0 - (moldura.right - moldura.left),
        externo.1 - (moldura.bottom - moldura.top),
    );
    let h = janela_com(Launch {
        animations: Some(false),
        ..Launch::background(FORA_DA_TELA, cliente)
    });
    let inicial = estado(&h);
    if inicial.dpi == 96 {
        let mut janela = RECT::default();
        unsafe {
            let _ = GetWindowRect(h.main, &mut janela);
        }
        assert_eq!(
            (janela.right - janela.left, janela.bottom - janela.top),
            externo,
            "mesma moldura da janela salva pelo usuário"
        );
    } else {
        eprintln!(
            "aviso: o monitor de teste está a {} dpi; a janela do usuário é de 96 dpi",
            inicial.dpi
        );
    }
    clicar(&h, Button::Three);
    let atual = estado(&h);
    let plot = atual.layout.plot;
    eprintln!(
        "janela do usuário: cliente {cliente:?}, conteúdo {}x{}, painel {:?}, plot {plot:?}",
        atual.layout.width, atual.layout.height, atual.layout.graph
    );
    assert!(
        !atual.layout.graph.is_empty(),
        "o 3×3 tem painel na janela do usuário"
    );
    assert!(plot.h >= 70, "plot com {} px de altura", plot.h);
    assert!(visivel(h.graph), "gráfico visível");
    assert_eq!(posicao_no_canvas(&h, h.graph), plot);
    clicar(&h, Button::Example);
    let resolvido = estado(&h);
    let esperado = repouso(&exemplo(3).sketch(), &resolvido, false);
    assert_eq!(
        resolvido.graph_scene,
        graph::describe(&esperado),
        "a cena desenhada é a do núcleo compartilhado"
    );
    let planos = esperado
        .iter()
        .filter(|item| {
            matches!(item.ink, Ink::Row(_)) && matches!(item.shape, Shape::Polygon { .. })
        })
        .count();
    assert_eq!(planos, 3, "três planos na cena");
    assert_eq!(texto(h.helper), LEGENDA_3X3);
    let imagem = capturar(h.main);
    let visivel_na_tela = plot.offset(-resolvido.scroll.0, -resolvido.scroll.1);
    assert!(
        visivel_na_tela.y >= 0
            && visivel_na_tela.bottom() <= imagem.altura
            && visivel_na_tela.right() <= imagem.largura,
        "plot inteiro na área cliente: {visivel_na_tela:?} em {}x{}",
        imagem.largura,
        imagem.altura
    );
    for (linha, pixels) in cores_dos_planos(&imagem, visivel_na_tela)
        .into_iter()
        .enumerate()
    {
        assert!(
            pixels > 25,
            "plano da equação {} desenhado ({pixels})",
            linha + 1
        );
    }
    let (x, y) = centro_do_ponto(&esperado, visivel_na_tela);
    assert!(
        perto(imagem.cor(x, y), theme::INK, 30),
        "ponto da solução em ({x},{y}) {:?}",
        imagem.cor(x, y)
    );
    fechar(&h);
}

#[test]
fn modo_pensar_e_valores_enormes_escondem_o_desenho() {
    let h = janela_animada(false);
    clicar(&h, Button::Example);
    clicar(&h, Button::Think);
    assert!(estado(&h).thinking);
    assert!(!visivel(h.graph), "o gráfico some no Modo pensar");
    clicar(&h, Button::Back);
    assert!(visivel(h.graph));
    escrever(&h, 0, 0, "1");
    escrever(&h, 0, 1, "1");
    escrever(&h, 0, 3, "1e200");
    assert_eq!(texto(h.helper), graph::HUGE);
    let atual = estado(&h);
    assert!(!atual.graph_scene.is_empty());
    assert!(
        !atual
            .graph_scene
            .split(';')
            .any(|item| item.starts_with('r') || item.starts_with('o')),
        "valores enormes não desenham retas nem ponto: {}",
        atual.graph_scene
    );
    fechar(&h);
}

#[test]
fn cores_alfa_e_ancoras_do_desenho() {
    let paleta = Palette::dark();
    let direta = plot::tint(Ink::Row(1), 0.5, &paleta, false).expect("cor");
    assert_eq!(
        (direta.r, direta.g, direta.b),
        (theme::MINT.r, theme::MINT.g, theme::MINT.b)
    );
    assert_eq!(direta.a, 128, "Direct2D usa o alfa");
    assert_eq!(plot::tint(Ink::Row(1), 0.0, &paleta, false), None);
    assert_eq!(
        plot::tint(Ink::Halo, 0.2, &paleta, true),
        Some(theme::ACCENT.mix(paleta.graph_fill, 20)),
        "no GDI a cor é misturada com o fundo"
    );
    assert_eq!(
        plot::tint(Ink::Halo, 0.09, &paleta, true),
        None,
        "no GDI, alfa abaixo de 0,1 é pulado"
    );
    assert_eq!(
        plot::tint(Ink::Point, 1.0, &paleta, true),
        Some(paleta.graph_point)
    );
    assert!(plot::rounded(Ink::Row(0)) && plot::rounded(Ink::Guide) && plot::rounded(Ink::Meet));
    assert!(!plot::rounded(Ink::Grid) && !plot::rounded(Ink::Axis));
    let area = Rect::new(10, 20, 100, 50);
    assert_eq!(
        plot::anchored(
            Pt::new(50.0, 25.0),
            Anchor::Middle,
            Anchor::Middle,
            (20, 14),
            area
        ),
        (10 + 40, 20 + 18)
    );
    assert_eq!(
        plot::anchored(
            Pt::new(97.0, 10.0),
            Anchor::End,
            Anchor::End,
            (20, 14),
            area
        ),
        (10 + 77, 20),
        "texto que sairia por cima volta para dentro"
    );
    assert_eq!(
        plot::anchored(
            Pt::new(95.0, 45.0),
            Anchor::Start,
            Anchor::Start,
            (20, 14),
            area
        ),
        (10 + 80, 20 + 36),
        "texto que sairia pela direita e por baixo volta para dentro"
    );
    let centros: Vec<f32> = dot_centers(20, 8.0).collect();
    assert_eq!(centros, [4.0, 12.0], "pontinhos enquanto cabem na área");
    assert_eq!(dot_centers(3, 8.0).count(), 0);
    let item = |shape: Shape, ink: Ink, alpha: f64| Item { shape, ink, alpha };
    let disco = || Shape::Disc {
        center: Pt::new(5.0, 5.0),
        radius: 3.0,
    };
    let triangulo = vec![Pt::new(0.0, 0.0), Pt::new(4.0, 0.0), Pt::new(0.0, 4.0)];
    assert!(plot::translucent_fill(&item(disco(), Ink::Halo, 0.2)));
    assert!(plot::translucent_fill(&item(
        Shape::Polygon {
            points: triangulo.clone()
        },
        Ink::Row(2),
        0.26
    )));
    assert!(!plot::translucent_fill(&item(disco(), Ink::Point, 1.0)));
    assert!(!plot::translucent_fill(&item(
        Shape::Ring {
            center: Pt::new(5.0, 5.0),
            radius: 6.0,
            width: 1.5
        },
        Ink::Halo,
        0.4
    )));
    assert!(!plot::translucent_fill(&item(
        Shape::Outline {
            points: triangulo,
            width: 1.25
        },
        Ink::Row(0),
        0.95
    )));
    assert_eq!(plot::label_band(1.0), graph::TEXT_HEIGHT as i32);
    assert_eq!(plot::label_band(1.25), 18);
    assert_eq!(plot::label_band(1.5), 21);
    assert_eq!(
        plot::label_offset(14, 8, 14),
        -3,
        "as maiúsculas ficam centradas na faixa de 14 px"
    );
}

#[test]
fn texto_que_nao_cabe_aperta_as_linhas_e_depois_usa_reticencias() {
    let medir = |texto: &str| texto.chars().count() as i32 * 10;
    let metrica = TextMetrics {
        height: 18,
        ascent: 14,
        descent: 4,
        external_leading: 0,
    };
    let frase = "aa bb cc dd ee ff gg hh";
    let planejar = |altura: i32| {
        gfx::plan_text(
            frase,
            Rect::new(0, 0, 88, altura),
            Align::Left,
            VAlign::Center,
            metrica,
            16,
            &medir,
        )
    };
    let posicoes = |linhas: &[PlacedLine]| -> Vec<(String, i32)> {
        linhas
            .iter()
            .map(|linha| (linha.text.clone(), linha.y))
            .collect()
    };
    let par = |texto: &str, y: i32| (texto.to_string(), y);
    assert_eq!(
        posicoes(&planejar(54)),
        [par("aa bb cc", 0), par("dd ee ff", 18), par("gg hh", 36)],
        "com espaço, o passo normal"
    );
    assert_eq!(
        posicoes(&planejar(53)),
        [par("aa bb cc", 0), par("dd ee ff", 17), par("gg hh", 34)],
        "faltando 1 px, as linhas se aproximam sem cortar nada"
    );
    assert_eq!(
        posicoes(&planejar(43)),
        [par("aa bb cc", 3), par("dd ee f\u{2026}", 21)],
        "sem espaço nem com o passo da fonte, a última linha termina em reticências"
    );
    assert_eq!(
        posicoes(&planejar(10)),
        [par("aa bb c\u{2026}", 0)],
        "uma linha só também avisa que há mais texto"
    );
    assert_eq!(
        posicoes(&gfx::plan_text(
            "curto",
            Rect::new(0, 0, 88, 30),
            Align::Left,
            VAlign::Center,
            metrica,
            16,
            &medir
        )),
        [par("curto", 6)]
    );
}

#[test]
fn legenda_longa_do_3x3_aparece_inteira_na_janela_padrao() {
    for renderer in [Renderer::Direct2D, Renderer::Gdi] {
        let h = janela_com(Launch {
            renderer,
            animations: Some(false),
            ..Launch::background(FORA_DA_TELA, CLIENTE)
        });
        clicar(&h, Button::Three);
        preencher_3x3(&h, &GRANDES);
        let atual = estado(&h);
        let legenda = atual.layout.helper;
        let frase = texto(h.helper);
        assert!(
            frase.starts_with("Os três planos se cruzam em (")
                && frase.ends_with("esse ponto é a solução do sistema."),
            "{frase}"
        );
        let (linhas, metrica, _) = plano_da_legenda(&h, &atual);
        let necessarias = linhas_necessarias(&atual, &frase, metrica);
        assert!(
            necessarias >= 3,
            "premissa: a frase longa quebra em {necessarias} linhas"
        );
        assert!(
            legenda.h >= necessarias as i32 * metrica.height,
            "a legenda do 3×3 ({} px) comporta {necessarias} linhas de {} px",
            legenda.h,
            metrica.height
        );
        assert_eq!(linhas.len(), necessarias, "{linhas:?}");
        let juntas: Vec<&str> = linhas.iter().map(|linha| linha.text.as_str()).collect();
        assert_eq!(
            juntas.join(" ").replace('\u{a0}', " "),
            frase,
            "nenhuma palavra se perde"
        );
        let passo = linhas[1].y - linhas[0].y;
        assert_eq!(
            passo, metrica.height,
            "as linhas mantêm o passo normal da fonte"
        );
        assert!(
            linhas
                .iter()
                .all(|linha| linha.y >= 0 && linha.y + metrica.height <= legenda.h),
            "todas as linhas cabem na legenda: {linhas:?}"
        );
        let imagem = capturar(h.main);
        let regiao = legenda.offset(-atual.scroll.0, -atual.scroll.1);
        let tinta = linhas_com_tinta(&imagem, regiao, theme::MUTED);
        let altura = tinta
            .last()
            .zip(tinta.first())
            .map_or(0, |(fim, inicio)| fim - inicio);
        assert!(
            altura >= (necessarias as i32 - 1) * passo,
            "{renderer:?}: {necessarias} linhas desenhadas ({altura} px de tinta)"
        );
        fechar(&h);
    }
}

fn linhas_necessarias(atual: &Diagnostics, frase: &str, metrica: TextMetrics) -> usize {
    let motor = Engine::new(atual.dpi).expect("motor de desenho");
    let medir = |linha: &str| motor.measure(linha, Font::Small);
    let (esquerda, direita) = sistemas_lineares::text::margins_for(metrica);
    wrap(
        &sistemas_lineares::text::keep_groups(frase),
        atual.layout.helper.w - esquerda - direita,
        &medir,
    )
    .len()
}

#[test]
fn legenda_longa_cabe_inteira_ate_na_janela_minima() {
    for cliente in [(640, 480), (1057, 797)] {
        let h = janela_com(Launch {
            animations: Some(false),
            ..Launch::background(FORA_DA_TELA, cliente)
        });
        clicar(&h, Button::Three);
        preencher_3x3(&h, &GRANDES);
        let atual = estado(&h);
        assert!(
            !atual.layout.graph.is_empty(),
            "{cliente:?}: o painel continua lá"
        );
        let frase = texto(h.helper);
        let (linhas, metrica, _) = plano_da_legenda(&h, &atual);
        let necessarias = linhas_necessarias(&atual, &frase, metrica);
        assert_eq!(
            linhas.len(),
            necessarias,
            "{cliente:?}: todas as linhas aparecem, sem reticências: {linhas:?}"
        );
        assert!(
            linhas.iter().all(|linha| !linha.text.ends_with('\u{2026}')),
            "{cliente:?}: {linhas:?}"
        );
        let juntas: Vec<&str> = linhas.iter().map(|linha| linha.text.as_str()).collect();
        assert_eq!(
            juntas.join(" ").replace('\u{a0}', " "),
            frase,
            "{cliente:?}: nenhuma palavra se perde"
        );
        assert!(
            linhas
                .iter()
                .all(|linha| linha.y >= 0 && linha.y + metrica.height <= atual.layout.helper.h),
            "{cliente:?}: {linhas:?}"
        );
        assert_eq!(
            nome_acessivel(h.helper),
            frase,
            "o leitor de tela recebe a frase inteira"
        );
        assert_eq!(descricao_acessivel(h.graph), frase);
        fechar(&h);
    }
}

#[test]
fn sem_alfa_o_gdi_nao_cobre_o_cruzamento_nem_os_planos() {
    let paleta = Palette::dark();
    for renderer in [Renderer::Direct2D, Renderer::Gdi] {
        let h = janela_com(Launch {
            renderer,
            animations: Some(false),
            ..Launch::background(FORA_DA_TELA, CLIENTE)
        });
        clicar(&h, Button::Example);
        let atual = estado(&h);
        let plot = atual.layout.plot;
        let esperado = repouso(&exemplo(2).sketch(), &atual, false);
        let centro = esperado
            .iter()
            .find_map(|item| match (&item.shape, item.ink) {
                (Shape::Disc { center, .. }, Ink::Point) => Some(*center),
                _ => None,
            })
            .expect("ponto da solução");
        let centro = (f64::from(plot.x) + centro.x, f64::from(plot.y) + centro.y);
        let unidade = f64::from(dpi_scale(atual.dpi));
        let raios = (7.0 * unidade, 10.0 * unidade);
        let imagem = capturar(h.main);
        let lilas = no_anel(&imagem, centro, raios, theme::ACCENT);
        let menta = no_anel(&imagem, centro, raios, theme::MINT);
        assert!(
            lilas > 0 && menta > 0,
            "{renderer:?}: as duas retas continuam visíveis em volta do ponto ({lilas}, {menta})"
        );
        if renderer == Renderer::Gdi {
            let brilho = plot::tint(Ink::Halo, 0.2, &paleta, true).expect("cor do brilho");
            assert_eq!(
                imagem.conta(area(plot), |cor| cor == brilho),
                0,
                "sem alfa, o brilho viraria um disco opaco"
            );
        }
        clicar(&h, Button::Three);
        clicar(&h, Button::Example);
        let atual = estado(&h);
        let plot = atual.layout.plot;
        let imagem = capturar(h.main);
        for (linha, cor) in [theme::ACCENT, theme::MINT, theme::PEACH]
            .into_iter()
            .enumerate()
        {
            let contorno = imagem.conta(area(plot), |pixel| perto(pixel, cor, 60));
            eprintln!("{renderer:?}: plano {} com {contorno} pixels", linha + 1);
            assert!(
                contorno > 20,
                "{renderer:?}: contorno do plano {} visível ({contorno})",
                linha + 1
            );
            if renderer == Renderer::Gdi {
                let opaco = plot::tint(Ink::Row(linha), 0.26, &paleta, true).expect("cor");
                assert_eq!(
                    imagem.conta(area(plot), |pixel| pixel == opaco),
                    0,
                    "sem alfa, o plano {} não vira uma placa opaca",
                    linha + 1
                );
            }
        }
        fechar(&h);
    }
}

#[test]
fn temporizador_so_anda_com_o_grafico_a_vista() {
    let h = janela_com(Launch {
        animations: Some(true),
        ..Launch::background(FORA_DA_TELA, (640, 480))
    });
    clicar(&h, Button::Three);
    assert!(
        !estado(&h).layout.plot.is_empty(),
        "na janela mínima, o 3×3 tem painel"
    );
    for (coluna, valor) in ["1", "1", "1", "6"].into_iter().enumerate() {
        escrever(&h, 0, coluna, valor);
    }
    let na_minima = estado(&h);
    assert!(
        na_minima.graph_timer && na_minima.graph_busy,
        "com o painel à vista, o plano novo anima"
    );
    assert!(esperar_grafico(&h), "e a animação termina sozinha");
    fechar(&h);

    let h = janela_animada(true);
    clicar(&h, Button::Example);
    assert!(estado(&h).graph_timer, "o exemplo anima");
    clicar(&h, Button::Think);
    let pensando = estado(&h);
    assert!(pensando.thinking && !visivel(h.graph));
    assert!(
        !pensando.graph_timer && !pensando.graph_busy,
        "o Modo pensar esconde o gráfico e encerra a animação"
    );
    clicar(&h, Button::Back);
    let de_volta = estado(&h);
    assert!(visivel(h.graph) && !de_volta.graph_timer);
    assert_eq!(
        de_volta.graph_scene,
        graph::describe(&repouso(&exemplo(2).sketch(), &de_volta, false)),
        "ao voltar, o gráfico já está em repouso"
    );

    clicar(&h, Button::Three);
    clicar(&h, Button::Example);
    assert!(estado(&h).graph_timer, "o exemplo 3×3 anima");
    redimensionar(&h, 640, 480);
    let encolhida = estado(&h);
    assert!(
        !encolhida.layout.plot.is_empty() && visivel(h.graph),
        "encolher até a janela mínima mantém o painel do 3×3"
    );
    assert_eq!(
        encolhida.graph_timer, encolhida.graph_busy,
        "o temporizador segue enquanto houver animação"
    );
    redimensionar(&h, CLIENTE.0, CLIENTE.1);
    let restaurada = estado(&h);
    assert!(visivel(h.graph));
    assert_eq!(
        restaurada.graph_timer, restaurada.graph_busy,
        "de volta ao tamanho padrão, o temporizador acompanha a animação"
    );
    assert!(esperar_grafico(&h));

    escrever(&h, 0, 3, "7");
    assert!(estado(&h).graph_timer, "mudar um valor anima de novo");
    unsafe {
        let _ = ShowWindow(h.main, SW_SHOWMINNOACTIVE);
    }
    pump(h.main);
    assert!(unsafe { IsIconic(h.main) }.as_bool());
    assert!(
        !estado(&h).graph_timer,
        "minimizada, a janela não pinta quadros"
    );
    unsafe {
        let _ = ShowWindow(h.main, SW_SHOWNOACTIVATE);
    }
    pump(h.main);
    assert!(!unsafe { IsIconic(h.main) }.as_bool());
    let reaberta = estado(&h);
    assert_eq!(reaberta.graph_timer, reaberta.graph_busy);
    assert!(esperar_grafico(&h));
    fechar(&h);
}

#[test]
fn rotulos_usam_a_faixa_do_nucleo_e_nao_encostam_no_desenho() {
    for renderer in [Renderer::Direct2D, Renderer::Gdi] {
        let h = janela_com(Launch {
            renderer,
            animations: Some(false),
            ..Launch::background(FORA_DA_TELA, CLIENTE)
        });
        clicar(&h, Button::Three);
        clicar(&h, Button::Example);
        let atual = estado(&h);
        let plot = atual.layout.plot;
        let itens = repouso(&exemplo(3).sketch(), &atual, false);
        let rotulo = itens
            .iter()
            .find_map(|item| match &item.shape {
                Shape::Text { at, text, .. } if text == "y" => Some(*at),
                _ => None,
            })
            .expect("rótulo do eixo y");
        let vertice = itens
            .iter()
            .filter(|item| item.ink == Ink::Edge)
            .filter_map(|item| match &item.shape {
                Shape::Outline { points, .. } => Some(points.clone()),
                _ => None,
            })
            .flatten()
            .map(|ponto| ponto.y)
            .fold(f64::INFINITY, f64::min);
        let imagem = capturar(h.main);
        let x = plot.x + rotulo.x.round() as i32;
        let topo_da_caixa = plot.y + vertice.floor() as i32;
        let regiao = Rect::new(x - 6, plot.y, 12, topo_da_caixa - plot.y + 4);
        let tinta = linhas_com_tinta(&imagem, regiao, theme::MUTED);
        let base = *tinta.last().expect("o rótulo y aparece");
        assert!(
            base < topo_da_caixa,
            "{renderer:?}: o y termina em {base} e a caixa começa em {topo_da_caixa}"
        );
        fechar(&h);
    }
}
