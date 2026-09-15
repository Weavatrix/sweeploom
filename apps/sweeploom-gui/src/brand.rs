//! Claude / Codex / Cursor marks copied from GrantTap provider artwork.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use sweeploom_core::SessionKind;

use crate::icons::{self, Glyph};
use crate::theme;

/// Paint a GrantTap provider mark when we know the tool.
pub fn show_tool(ui: &mut egui::Ui, tool: &str, size: f32) {
    if let Some((id, bytes)) = tool_png(tool) {
        show_png(ui, id, bytes, size);
    } else {
        icons::show(ui, app_glyph(tool), size, theme::accent());
    }
}

/// History / listing mark: session kind when known, otherwise the title.
pub fn show_group(ui: &mut egui::Ui, title: &str, kind: Option<SessionKind>, size: f32) {
    match kind {
        Some(kind) => show_session(ui, kind, title, size),
        None => show_tool(ui, title, size),
    }
}

/// Session row mark: provider art or a kind glyph.
pub fn show_session(ui: &mut egui::Ui, kind: SessionKind, title: &str, size: f32) {
    if let Some(tool) = session_tool(kind, title) {
        show_tool(ui, tool, size);
        return;
    }
    icons::show(ui, session_glyph(kind, title), size, theme::accent());
}

fn session_tool(kind: SessionKind, title: &str) -> Option<&'static str> {
    match kind {
        SessionKind::ClaudeCode => Some("claude"),
        SessionKind::Codex => Some("codex"),
        SessionKind::Cursor => Some("cursor"),
        _ => named_tool(title),
    }
}

fn named_tool(title: &str) -> Option<&'static str> {
    let lower = title.to_ascii_lowercase();
    if lower.contains("claude") {
        Some("claude")
    } else if lower.contains("codex") {
        Some("codex")
    } else if lower.contains("cursor") {
        Some("cursor")
    } else if is_vscode(title) {
        Some("code")
    } else {
        None
    }
}

fn is_vscode(title: &str) -> bool {
    let file = title.rsplit(['/', '\\']).next().unwrap_or(title);
    let stem = file
        .strip_suffix(".exe")
        .or_else(|| file.strip_suffix(".EXE"))
        .unwrap_or(file)
        .trim();
    let lower = stem.to_ascii_lowercase();
    lower == "code" || lower == "vscode" || lower.starts_with("code -")
}

fn session_glyph(kind: SessionKind, title: &str) -> Glyph {
    match kind {
        SessionKind::Browser => Glyph::Browser,
        SessionKind::Terminal => Glyph::Sessions,
        SessionKind::Mcp
        | SessionKind::ClaudeCode
        | SessionKind::Codex
        | SessionKind::Cursor
        | SessionKind::OpenCode
        | SessionKind::Gemini
        | SessionKind::Grok => Glyph::Ai,
        SessionKind::Build | SessionKind::TestRunner => Glyph::Projects,
        SessionKind::DevServer | SessionKind::LanguageServer => Glyph::Cpu,
        SessionKind::Container => Glyph::Disk,
        SessionKind::GenericApp | SessionKind::Unknown => app_glyph(title),
    }
}

fn app_glyph(title: &str) -> Glyph {
    let lower = title.to_ascii_lowercase();
    if ["msedge", "chrome", "firefox", "brave", "opera"]
        .iter()
        .any(|n| lower.contains(n))
    {
        Glyph::Browser
    } else if lower.contains("explorer") {
        Glyph::Explorer
    } else if is_vscode(title) || ["devenv", "windsurf"].iter().any(|n| lower.contains(n)) {
        Glyph::Projects
    } else if ["pwsh", "powershell", "cmd", "windowsterminal", "wt"]
        .iter()
        .any(|n| lower.contains(n))
    {
        Glyph::Sessions
    } else {
        Glyph::Overview
    }
}

fn tool_png(tool: &str) -> Option<(&'static str, &'static [u8])> {
    let lower = tool.to_ascii_lowercase();
    if lower.contains("claude") {
        Some((
            "claude",
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/claude.png")),
        ))
    } else if lower.contains("codex") {
        Some((
            "codex",
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/codex.png")),
        ))
    } else if lower.contains("cursor") {
        Some((
            "cursor",
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/cursor.png")),
        ))
    } else if is_vscode(tool) {
        Some((
            "code",
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/code.png")),
        ))
    } else {
        None
    }
}

fn show_png(ui: &mut egui::Ui, id: &str, bytes: &[u8], size: f32) {
    let texture = texture(ui.ctx(), id, bytes);
    let side = size.max(12.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    let radius = side * 0.22;
    ui.painter()
        .rect_filled(rect, radius, egui::Color32::from_rgb(236, 236, 240));
    egui::Image::new((texture.id(), egui::vec2(side, side)))
        .fit_to_exact_size(egui::vec2(side, side))
        .corner_radius(radius)
        .paint_at(ui, rect);
}

fn texture(ctx: &egui::Context, id: &str, bytes: &[u8]) -> TextureHandle {
    let key = egui::Id::new(("sweeploom-brand", id));
    if let Some(handle) = ctx.data(|data| data.get_temp::<TextureHandle>(key)) {
        return handle;
    }
    let decoded = image::load_from_memory(bytes)
        .map(|img| img.to_rgba8())
        .unwrap_or_else(|_| image::RgbaImage::new(1, 1));
    let size = [decoded.width() as usize, decoded.height() as usize];
    let image = ColorImage::from_rgba_unmultiplied(size, &decoded);
    let handle = ctx.load_texture(id, image, TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(key, handle.clone()));
    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_session_uses_claude_art() {
        assert_eq!(
            session_tool(SessionKind::ClaudeCode, "agent"),
            Some("claude")
        );
        assert_eq!(session_tool(SessionKind::Cursor, "agent"), Some("cursor"));
        assert_eq!(named_tool("Cursor.exe"), Some("cursor"));
        assert_eq!(named_tool("Code"), Some("code"));
        assert_eq!(named_tool("Codex"), Some("codex"));
        assert!(!is_vscode("Codex"));
        assert!(is_vscode("Code.exe"));
    }

    #[test]
    fn edge_uses_browser_glyph() {
        assert_eq!(app_glyph("msedge"), Glyph::Browser);
        assert_eq!(app_glyph("explorer"), Glyph::Explorer);
        assert_eq!(app_glyph("Code"), Glyph::Projects);
    }
}
