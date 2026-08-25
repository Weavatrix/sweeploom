//! Optional desktop tray. Unsupported platforms return `None`.

/// Commands from the tray menu or icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayCommand {
    /// Show the main window.
    Show,
    /// Quit the process.
    Quit,
}

/// True when a tray icon can be created on this OS.
#[must_use]
pub const fn is_supported() -> bool {
    cfg!(any(windows, target_os = "macos"))
}

#[cfg(any(windows, target_os = "macos"))]
mod native {
    use super::{TrayCommand, is_supported};
    use std::collections::VecDeque;
    use std::sync::{Mutex, OnceLock};
    use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
    use tray_icon::{
        Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    };

    struct Bridge {
        commands: Mutex<VecDeque<TrayCommand>>,
        show: Mutex<Option<MenuId>>,
        quit: Mutex<Option<MenuId>>,
    }

    fn bridge() -> &'static Bridge {
        static BRIDGE: OnceLock<Bridge> = OnceLock::new();
        BRIDGE.get_or_init(|| Bridge {
            commands: Mutex::new(VecDeque::new()),
            show: Mutex::new(None),
            quit: Mutex::new(None),
        })
    }

    fn push(command: TrayCommand) {
        if let Ok(mut queue) = bridge().commands.lock() {
            queue.push_back(command);
        }
    }

    fn remember_ids(show: MenuId, quit: MenuId) {
        if let Ok(mut slot) = bridge().show.lock() {
            *slot = Some(show);
        }
        if let Ok(mut slot) = bridge().quit.lock() {
            *slot = Some(quit);
        }
    }

    fn clear_ids() {
        if let Ok(mut slot) = bridge().show.lock() {
            *slot = None;
        }
        if let Ok(mut slot) = bridge().quit.lock() {
            *slot = None;
        }
    }

    fn menu_command(id: &MenuId) -> Option<TrayCommand> {
        let show = bridge().show.lock().ok()?;
        let quit = bridge().quit.lock().ok()?;
        if show.as_ref() == Some(id) {
            return Some(TrayCommand::Show);
        }
        if quit.as_ref() == Some(id) {
            return Some(TrayCommand::Quit);
        }
        None
    }

    /// Keeps the tray icon and menu alive.
    pub struct TrayIconHandle {
        _icon: TrayIcon,
        _show: MenuItem,
        _quit: MenuItem,
    }

    impl Drop for TrayIconHandle {
        fn drop(&mut self) {
            clear_ids();
        }
    }

    /// Build a tray icon. `None` when the OS refuses it.
    pub fn create() -> Option<TrayIconHandle> {
        if !is_supported() {
            return None;
        }
        let menu = Menu::new();
        let show = MenuItem::new("Open SweepLoom", true, None);
        let quit = MenuItem::new("Quit", true, None);
        menu.append(&show).ok()?;
        menu.append(&quit).ok()?;
        remember_ids(show.id().clone(), quit.id().clone());
        let icon = make_icon()?;
        let tray = TrayIconBuilder::new()
            .with_tooltip("SweepLoom")
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .build()
            .ok()?;
        Some(TrayIconHandle {
            _icon: tray,
            _show: show,
            _quit: quit,
        })
    }

    /// Drain pending tray events.
    pub fn poll(_handle: &TrayIconHandle) -> Option<TrayCommand> {
        bridge().commands.lock().ok()?.pop_front()
    }

    fn make_icon() -> Option<Icon> {
        let n = 32_u32;
        let mut rgba = vec![0_u8; (n * n * 4) as usize];
        for y in 0..n {
            for x in 0..n {
                let edge = x.min(y).min(n - 1 - x).min(n - 1 - y);
                if edge < 3 {
                    continue;
                }
                let i = ((y * n + x) * 4) as usize;
                rgba[i] = 196;
                rgba[i + 1] = 140;
                rgba[i + 2] = 64;
                rgba[i + 3] = 255;
            }
        }
        Icon::from_rgba(rgba, n, n).ok()
    }

    /// Wake the GUI thread on real actions. Hover/move must not repaint.
    pub fn install_wake(ctx: eframe::egui::Context) {
        let paint = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |event| {
            let open = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            );
            if open {
                push(TrayCommand::Show);
                ctx.request_repaint();
            }
        }));
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            if let Some(command) = menu_command(event.id()) {
                push(command);
                paint.request_repaint();
            }
        }));
    }
}

#[cfg(any(windows, target_os = "macos"))]
pub use native::{TrayIconHandle, create, install_wake, poll};

#[cfg(not(any(windows, target_os = "macos")))]
/// Placeholder when the OS has no tray backend.
pub struct TrayIconHandle;

#[cfg(not(any(windows, target_os = "macos")))]
/// Always `None` on this OS.
pub fn create() -> Option<TrayIconHandle> {
    None
}

#[cfg(not(any(windows, target_os = "macos")))]
/// No events.
pub fn poll(_handle: &TrayIconHandle) -> Option<TrayCommand> {
    None
}

#[cfg(not(any(windows, target_os = "macos")))]
/// No-op.
pub fn install_wake(_ctx: eframe::egui::Context) {}
