#[cfg(windows)]
pub mod clipboard;
#[cfg(windows)]
pub mod fonts;
#[cfg(windows)]
pub mod gfx;
#[cfg(windows)]
pub mod install;
#[cfg(windows)]
pub mod plot;
#[cfg(windows)]
pub mod registry;
#[cfg(windows)]
pub mod settings;
#[cfg(windows)]
pub mod window;

pub use sistemas_lineares_ui::{graph, icons, layout, model, text, theme, thinking};

#[cfg(windows)]
pub use window::{
    Button, Diagnostics, Handles, Launch, Renderer, create, diagnostics, dispatch,
    force_animations, force_high_contrast, message_loop, pump, run, window_text,
};
