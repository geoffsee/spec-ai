use crate::models::ChatRole;
use crate::state::{AppState, PanelFocus};
use spec_ai_tui::{
    buffer::Buffer,
    geometry::Rect,
    layout::{Constraint, Layout},
    style::{wrap_text, Color, Line, Span, Style},
    widget::{
        builtin::{Block, Editor, SlashCommand, SlashMenu, StatusBar, StatusSection},
        StatefulWidget, Widget,
    },
};

pub fn render(state: &AppState, area: Rect, buf: &mut Buffer) {
    let layout = Layout::vertical()
        .constraints([
            Constraint::Fill(1),
            Constraint::Fixed(6),
            Constraint::Fixed(3),
            Constraint::Fixed(1),
        ])
        .split(area);

    render_chat(state, layout[0], buf);
    render_input(state, layout[1], buf);
    render_reasoning(state, layout[2], buf);
    render_status(state, layout[3], buf);
}

fn render_chat(state: &AppState, area: Rect, buf: &mut Buffer) {
    let border_style = if state.focus == PanelFocus::Chat {
        Style::new().fg(Color::Cyan)
    } else {
        Style::new().fg(Color::DarkGrey)
    };

    let title = match &state.active_agent {
        Some(agent) => format!("Conversation · Active agent: {}", agent),
        None => "Conversation".to_string(),
    };

    let block = Block::bordered()
        .title(title)
        .border_style(border_style.clone());
    Widget::render(&block, area, buf);

    let inner = block.inner(area);
    if inner.is_empty() {
        return;
    }

    let content_width = inner.width.saturating_sub(1) as usize;
    let mut lines: Vec<Line> = Vec::new();

    for message in &state.messages {
        let (style, label) = role_style(&message.role);
        lines.push(Line::from_spans([
            Span::styled(
                format!("[{}] ", message.timestamp),
                Style::new().fg(Color::DarkGrey),
            ),
            Span::styled(format!("{}:", label), style),
        ]));

        let prefix = "  ";
        for content_line in message.content.lines() {
            let wrapped = wrap_text(&format!("{prefix}{content_line}"), content_width, prefix);
            for wrapped_line in wrapped {
                lines.push(Line::raw(wrapped_line));
            }
        }

        lines.push(Line::empty());
    }

    let visible_height = inner.height as usize;
    let total_lines = lines.len();
    let scroll = state.scroll_offset as usize;
    let start = if total_lines > visible_height + scroll {
        total_lines - visible_height - scroll
    } else {
        0
    };
    let end = (start + visible_height).min(total_lines);

    for (i, line) in lines[start..end].iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.bottom() {
            break;
        }
        buf.set_line(inner.x, y, line);
    }

    if total_lines > visible_height {
        let scrollbar_height = inner.height.saturating_sub(1);
        let thumb_pos = if total_lines > 0 {
            ((start as u32 * scrollbar_height as u32) / total_lines as u32) as u16
        } else {
            0
        };

        for y in 0..scrollbar_height {
            let char = if y == thumb_pos { "█" } else { "░" };
            buf.set_string(
                inner.right().saturating_sub(1),
                inner.y + y,
                char,
                Style::new().fg(Color::DarkGrey),
            );
        }
    }
}

fn render_input(state: &AppState, area: Rect, buf: &mut Buffer) {
    let border_style = if state.focus == PanelFocus::Input {
        Style::new().fg(Color::Cyan)
    } else {
        Style::new().fg(Color::DarkGrey)
    };

    let block = Block::bordered().title("Input").border_style(border_style);
    Widget::render(&block, area, buf);

    let inner = block.inner(area);
    if inner.is_empty() {
        return;
    }

    let help_text = if state.editor.show_slash_menu {
        "Tab: autocomplete | ↑/↓: select | Enter: run"
    } else {
        "Ctrl+C: quit | Ctrl+L: clear | / commands | Alt+b/f: word nav"
    };
    buf.set_string(
        inner.x,
        inner.y,
        help_text,
        Style::new().fg(Color::DarkGrey),
    );

    buf.set_string(inner.x, inner.y + 1, "▸ ", Style::new().fg(Color::Green));

    let editor_height = inner.height.saturating_sub(1);
    let editor_area = Rect::new(
        inner.x + 2,
        inner.y + 1,
        inner.width.saturating_sub(2),
        editor_height,
    );
    let editor = Editor::new()
        .placeholder("Ask spec-ai or run /commands...")
        .style(Style::new().fg(Color::White));

    let mut editor_state = state.editor.clone();
    editor.render(editor_area, buf, &mut editor_state);

    if state.editor.show_slash_menu {
        let filtered_commands: Vec<SlashCommand> = state
            .slash_commands
            .iter()
            .filter(|cmd| cmd.matches(&state.editor.slash_query))
            .cloned()
            .collect();

        if !filtered_commands.is_empty() {
            let menu = SlashMenu::new()
                .commands(filtered_commands)
                .query(&state.editor.slash_query);

            let menu_area = Rect::new(
                inner.x + 2,
                area.y,
                inner.width.saturating_sub(2).min(50),
                area.height,
            );

            let mut menu_state = state.slash_menu.clone();
            menu.render(menu_area, buf, &mut menu_state);
        }
    }
}

fn render_reasoning(state: &AppState, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered().title("Reasoning");
    Widget::render(&block, area, buf);

    let inner = block.inner(area);
    if inner.is_empty() {
        return;
    }

    let spinner_frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let spinner = if state.busy {
        spinner_frames[(state.tick / 2) as usize % spinner_frames.len()]
    } else {
        '◇'
    };

    let entries = if state.reasoning.is_empty() {
        vec!["Waiting for backend...".to_string()]
    } else {
        state.reasoning.clone()
    };

    for (idx, line) in entries.iter().take(inner.height as usize).enumerate() {
        let prefix = if idx == 0 {
            format!("{spinner} ")
        } else {
            "  ".to_string()
        };
        let rendered = format!("{prefix}{line}");
        buf.set_string(
            inner.x,
            inner.y + idx as u16,
            &rendered,
            Style::new().fg(Color::White),
        );
    }
}

fn render_status(state: &AppState, area: Rect, buf: &mut Buffer) {
    let mut left_sections = vec![StatusSection::new(&state.status)];
    if let Some(err) = &state.error {
        left_sections
            .push(StatusSection::new(format!("Error: {}", err)).style(Style::new().fg(Color::Red)));
    }

    let center_sections = if state.busy {
        vec![StatusSection::new("Working").style(Style::new().fg(Color::Yellow))]
    } else {
        vec![StatusSection::new("Idle").style(Style::new().fg(Color::Green))]
    };

    let right_sections = vec![
        StatusSection::new("Tab: scroll/chat"),
        StatusSection::new("Ctrl+C: quit"),
    ];

    let bar = StatusBar::new()
        .left(left_sections)
        .center(center_sections)
        .right(right_sections)
        .style(Style::new().bg(Color::DarkGrey).fg(Color::White));

    Widget::render(&bar, area, buf);
}

fn role_style(role: &ChatRole) -> (Style, String) {
    match role {
        ChatRole::User => (Style::new().fg(Color::Green).bold(), role.label()),
        ChatRole::Assistant => (Style::new().fg(Color::Cyan).bold(), role.label()),
        ChatRole::System => (Style::new().fg(Color::Yellow).bold(), role.label()),
        ChatRole::Agent(_) => (Style::new().fg(Color::Magenta).bold(), role.label()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spec_ai_tui::style::Modifier;

    #[test]
    fn role_style_user_returns_green() {
        let (style, label) = role_style(&ChatRole::User);
        assert_eq!(style.fg, Color::Green);
        assert_eq!(label, "User");
    }

    #[test]
    fn role_style_assistant_returns_cyan() {
        let (style, label) = role_style(&ChatRole::Assistant);
        assert_eq!(style.fg, Color::Cyan);
        assert_eq!(label, "Assistant");
    }

    #[test]
    fn role_style_system_returns_yellow() {
        let (style, label) = role_style(&ChatRole::System);
        assert_eq!(style.fg, Color::Yellow);
        assert_eq!(label, "System");
    }

    #[test]
    fn role_style_agent_returns_magenta() {
        let (style, label) = role_style(&ChatRole::Agent("test".to_string()));
        assert_eq!(style.fg, Color::Magenta);
        assert_eq!(label, "Agent test");
    }

    #[test]
    fn role_style_all_are_bold() {
        let roles = [
            ChatRole::User,
            ChatRole::Assistant,
            ChatRole::System,
            ChatRole::Agent("x".to_string()),
        ];
        for role in &roles {
            let (style, _) = role_style(role);
            assert!(
                style.modifier.contains(Modifier::BOLD),
                "Style for {:?} should be bold",
                role
            );
        }
    }
}
