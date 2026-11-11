use crate::registry::{self, Key};

pub const KEY: &str = r"Software\Lucas Lima\Sistemas Lineares";

const LEFT: &str = "Esquerda";
const TOP: &str = "Topo";
const WIDTH: &str = "Largura";
const HEIGHT: &str = "Altura";
const MAXIMIZED: &str = "Maximizada";
const DPI: &str = "Dpi";
const DIMENSION: &str = "Dimensao";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl Bounds {
    pub const fn right(self) -> i32 {
        self.left + self.width
    }

    pub const fn bottom(self) -> i32 {
        self.top + self.height
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Placement {
    pub bounds: Bounds,
    pub maximized: bool,
    pub dpi: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Saved {
    pub placement: Option<Placement>,
    pub dimension: Option<usize>,
}

fn valid_dpi(dpi: u32) -> bool {
    (48..=1920).contains(&dpi)
}

fn valid_size(value: i32) -> bool {
    (1..=32_767).contains(&value)
}

pub fn load(key: &str) -> Saved {
    let Some(key) = Key::open(key) else {
        return Saved::default();
    };
    let signed = |name: &str| key.dword(name).map(|value| value as i32);
    let placement = match (
        signed(LEFT),
        signed(TOP),
        signed(WIDTH),
        signed(HEIGHT),
        key.dword(DPI),
    ) {
        (Some(left), Some(top), Some(width), Some(height), Some(dpi))
            if valid_size(width)
                && valid_size(height)
                && valid_dpi(dpi)
                && left.checked_add(width).is_some()
                && top.checked_add(height).is_some() =>
        {
            Some(Placement {
                bounds: Bounds {
                    left,
                    top,
                    width,
                    height,
                },
                maximized: key.dword(MAXIMIZED) == Some(1),
                dpi,
            })
        }
        _ => None,
    };
    let dimension = key.dword(DIMENSION).and_then(|value| match value {
        2 => Some(2),
        3 => Some(3),
        _ => None,
    });
    Saved {
        placement,
        dimension,
    }
}

pub fn save(key: &str, saved: &Saved) -> bool {
    let Some(key) = Key::create(key) else {
        return false;
    };
    let mut written = true;
    if let Some(placement) = saved.placement {
        let bounds = placement.bounds;
        written &= key.set_dword(LEFT, bounds.left as u32);
        written &= key.set_dword(TOP, bounds.top as u32);
        written &= key.set_dword(WIDTH, bounds.width as u32);
        written &= key.set_dword(HEIGHT, bounds.height as u32);
        written &= key.set_dword(MAXIMIZED, u32::from(placement.maximized));
        written &= key.set_dword(DPI, placement.dpi);
    }
    if let Some(dimension) = saved.dimension {
        written &= key.set_dword(DIMENSION, dimension.clamp(2, 3) as u32);
    }
    written
}

pub fn erase(key: &str) -> bool {
    let removed = registry::delete_tree(key);
    if let Some(parent) = registry::parent(key) {
        registry::delete_if_empty(parent);
    }
    removed
}

pub fn fit(placement: Placement, work: Bounds, dpi: u32, minimum: (i32, i32)) -> Bounds {
    let scale = |value: i32| -> i32 {
        if placement.dpi == dpi || placement.dpi == 0 {
            value
        } else {
            ((i64::from(value) * i64::from(dpi) + i64::from(placement.dpi) / 2)
                / i64::from(placement.dpi))
            .clamp(1, 32_767) as i32
        }
    };
    let available_w = work.width.max(1);
    let available_h = work.height.max(1);
    let width = scale(placement.bounds.width)
        .max(minimum.0)
        .min(available_w);
    let height = scale(placement.bounds.height)
        .max(minimum.1)
        .min(available_h);
    let left = placement
        .bounds
        .left
        .min(work.right() - width)
        .max(work.left);
    let top = placement
        .bounds
        .top
        .min(work.bottom() - height)
        .max(work.top);
    Bounds {
        left,
        top,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placement(left: i32, top: i32, width: i32, height: i32, dpi: u32) -> Placement {
        Placement {
            bounds: Bounds {
                left,
                top,
                width,
                height,
            },
            maximized: false,
            dpi,
        }
    }

    #[test]
    fn mantem_janela_que_cabe_no_monitor() {
        let work = Bounds {
            left: 0,
            top: 0,
            width: 1920,
            height: 1040,
        };
        let fitted = fit(placement(100, 50, 1176, 849, 96), work, 96, (640, 480));
        assert_eq!(
            fitted,
            Bounds {
                left: 100,
                top: 50,
                width: 1176,
                height: 849
            }
        );
    }

    #[test]
    fn puxa_para_dentro_e_limita_ao_monitor() {
        let work = Bounds {
            left: 1920,
            top: 0,
            width: 1280,
            height: 984,
        };
        let fitted = fit(placement(2900, 700, 1176, 849, 96), work, 96, (640, 480));
        assert_eq!(
            fitted,
            Bounds {
                left: 1920 + 1280 - 1176,
                top: 984 - 849,
                width: 1176,
                height: 849
            }
        );
        let fitted = fit(placement(1920, 0, 4000, 3000, 96), work, 96, (640, 480));
        assert_eq!(fitted, work);
    }

    #[test]
    fn reescala_quando_o_dpi_muda() {
        let work = Bounds {
            left: 0,
            top: 0,
            width: 3840,
            height: 2100,
        };
        let fitted = fit(placement(10, 20, 1176, 849, 96), work, 144, (960, 720));
        assert_eq!((fitted.width, fitted.height), (1764, 1274));
        let fitted = fit(placement(10, 20, 600, 400, 96), work, 96, (640, 480));
        assert_eq!((fitted.width, fitted.height), (640, 480));
    }

    #[test]
    fn area_degenerada_nao_entra_em_panico() {
        let fitted = fit(
            placement(5, 5, 800, 600, 96),
            Bounds {
                left: 0,
                top: 0,
                width: 0,
                height: 0,
            },
            96,
            (640, 480),
        );
        assert_eq!((fitted.width, fitted.height), (1, 1));
        let fitted = fit(
            placement(i32::MAX - 10, i32::MIN + 10, 800, 600, 0),
            Bounds {
                left: -100,
                top: -100,
                width: 50,
                height: 50,
            },
            96,
            (640, 480),
        );
        assert!(fitted.width >= 1 && fitted.height >= 1);
    }
}
