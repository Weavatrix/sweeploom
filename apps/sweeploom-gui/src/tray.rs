//! Optional desktop tray. Unsupported platforms return `None`.

use crate::nav::Nav;

/// Commands from the tray menu or icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayCommand {
    /// Show the main window.
    Show,
    /// Hide the main window; keep the process.
    Hide,
    /// Flip visibility.
    Toggle,
    /// Show and jump to a screen.
    Open(Nav),
    /// Show Review and rebuild disk offers.
    Rebuild,
    /// Quit the process.
    Quit,
}

/// True when a tray icon can be created on this OS.
#[must_use]
pub const fn is_supported() -> bool {
    cfg!(any(windows, target_os = "macos"))
}

#[must_use]
fn command_from_id(id: &str) -> Option<TrayCommand> {
    Some(match id {
        "show" => TrayCommand::Show,
        "hide" => TrayCommand::Hide,
        "overview" => TrayCommand::Open(Nav::Overview),
        "sessions" => TrayCommand::Open(Nav::Sessions),
        "review" => TrayCommand::Open(Nav::Storage),
        "projects" => TrayCommand::Open(Nav::Projects),
        "browser" => TrayCommand::Open(Nav::Browser),
        "ai" => TrayCommand::Open(Nav::Ai),
        "explorer" => TrayCommand::Open(Nav::Explorer),
        "history" => TrayCommand::Open(Nav::History),
        "rebuild" => TrayCommand::Rebuild,
        "quit" => TrayCommand::Quit,
        _ => return None,
    })
}

#[cfg(any(windows, target_os = "macos"))]
mod native {
    use super::{TrayCommand, command_from_id, is_supported};
    use std::collections::VecDeque;
    use std::sync::{Mutex, OnceLock};
    use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{
        Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    };

    struct Bridge {
        commands: Mutex<VecDeque<TrayCommand>>,
        wake: OnceLock<eframe::egui::Context>,
    }

    fn bridge() -> &'static Bridge {
        static BRIDGE: OnceLock<Bridge> = OnceLock::new();
        BRIDGE.get_or_init(|| Bridge {
            commands: Mutex::new(VecDeque::new()),
            wake: OnceLock::new(),
        })
    }

    fn push(command: TrayCommand) {
        if let Ok(mut queue) = bridge().commands.lock() {
            queue.push_back(command);
        }
        if let Some(ctx) = bridge().wake.get() {
            ctx.request_repaint();
        }
    }

    fn click_opens(event: &TrayIconEvent) -> bool {
        matches!(
            event,
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            }
        )
    }

    /// Keeps the tray icon and menu alive.
    pub struct TrayIconHandle {
        _icon: TrayIcon,
        _items: Vec<MenuItem>,
        _separators: Vec<PredefinedMenuItem>,
    }

    fn append_item(menu: &Menu, items: &mut Vec<MenuItem>, id: &str, label: &str) -> Option<()> {
        let item = MenuItem::with_id(id, label, true, None);
        menu.append(&item).ok()?;
        items.push(item);
        Some(())
    }

    fn append_sep(menu: &Menu, seps: &mut Vec<PredefinedMenuItem>) -> Option<()> {
        let sep = PredefinedMenuItem::separator();
        menu.append(&sep).ok()?;
        seps.push(sep);
        Some(())
    }

    fn fill_menu(
        menu: &Menu,
        items: &mut Vec<MenuItem>,
        seps: &mut Vec<PredefinedMenuItem>,
    ) -> Option<()> {
        append_item(menu, items, "show", "Open SweepLoom")?;
        append_item(menu, items, "hide", "Hide to tray")?;
        append_sep(menu, seps)?;
        append_item(menu, items, "overview", "Overview")?;
        append_item(menu, items, "sessions", "Sessions")?;
        append_item(menu, items, "review", "Review")?;
        append_item(menu, items, "projects", "Projects")?;
        append_item(menu, items, "browser", "Browser")?;
        append_item(menu, items, "ai", "AI")?;
        append_item(menu, items, "explorer", "Explorer")?;
        append_item(menu, items, "history", "History")?;
        append_sep(menu, seps)?;
        append_item(menu, items, "rebuild", "Rebuild review")?;
        append_sep(menu, seps)?;
        append_item(menu, items, "quit", "Quit")?;
        Some(())
    }

    /// Build a tray icon. `None` when the OS refuses it.
    pub fn create() -> Option<TrayIconHandle> {
        if !is_supported() {
            return None;
        }
        let menu = Menu::new();
        let mut items = Vec::new();
        let mut separators = Vec::new();
        fill_menu(&menu, &mut items, &mut separators)?;
        let icon = make_icon()?;
        let tray = TrayIconBuilder::new()
            .with_tooltip("SweepLoom")
            .with_menu(Box::new(menu))
            .with_icon(icon)
            .with_menu_on_left_click(false)
            .build()
            .ok()?;
        Some(TrayIconHandle {
            _icon: tray,
            _items: items,
            _separators: separators,
        })
    }

    /// Drain one pending tray command on the GUI thread.
    pub fn poll() -> Option<TrayCommand> {
        bridge().commands.lock().ok()?.pop_front()
    }

    fn make_icon() -> Option<Icon> {
        let (rgba, width, height) = crate::mark::rgba(32);
        Icon::from_rgba(rgba, width, height).ok()
    }

    /// Forward tray/menu clicks into the GUI event loop.
    pub fn install_wake(ctx: eframe::egui::Context) {
        let _ = bridge().wake.set(ctx);
        TrayIconEvent::set_event_handler(Some(|event: TrayIconEvent| {
            if click_opens(&event) {
                push(TrayCommand::Toggle);
            }
        }));
        MenuEvent::set_event_handler(Some(|event: MenuEvent| {
            if let Some(command) = command_from_id(event.id().as_ref()) {
                push(command);
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
pub fn poll() -> Option<TrayCommand> {
    None
}

#[cfg(not(any(windows, target_os = "macos")))]
/// No-op.
pub fn install_wake(_ctx: eframe::egui::Context) {}

#[cfg(test)]
mod tests {
    use super::{TrayCommand, command_from_id};
    use crate::nav::Nav;

    #[test]
    fn tray_ids_map_to_commands() {
        assert_eq!(command_from_id("show"), Some(TrayCommand::Show));
        assert_eq!(command_from_id("hide"), Some(TrayCommand::Hide));
        assert_eq!(
            command_from_id("review"),
            Some(TrayCommand::Open(Nav::Storage))
        );
        assert_eq!(command_from_id("rebuild"), Some(TrayCommand::Rebuild));
        assert_eq!(command_from_id("quit"), Some(TrayCommand::Quit));
        assert_eq!(command_from_id("nope"), None);
    }
}
