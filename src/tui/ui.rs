use super::app::{App, AppMode, EditList, InputPurpose};
use crate::config;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(size);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main_chunks[1]);

    // Header
    let bound_name = app
        .bound_profile_name()
        .map(|s| s.as_str())
        .unwrap_or("None");
    let header_text = format!(" Dir: {} | Bound Profile: [{}] ", app.path, bound_name);
    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" yoink manage "),
        )
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(header, main_chunks[0]);

    // Left Pane (Profiles)
    let is_profile_focused = app.mode == AppMode::ProfileList;
    let items: Vec<ListItem> = app
        .profiles
        .iter()
        .map(|p| {
            let mut text = p.clone();
            if Some(p) == app.bound_profile_name() {
                text.push_str(" (bound)");
            }
            ListItem::new(text)
        })
        .collect();

    let profiles_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Profiles ")
                .border_style(Style::default().fg(if is_profile_focused {
                    Color::Cyan
                } else {
                    Color::DarkGray
                })),
        )
        .highlight_style(
            Style::default()
                .bg(if is_profile_focused {
                    Color::Cyan
                } else {
                    Color::DarkGray
                })
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(profiles_list, body_chunks[0], &mut app.profile_state);

    // Right Pane (Edit Mode)
    let is_edit_focused = app.mode == AppMode::EditProfile;

    // Split right pane to hold a settings header + list chunks
    let right_pane_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Settings Header
            Constraint::Min(0),    // Lists
        ])
        .split(body_chunks[1]);

    let edit_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(right_pane_chunks[1]);

    let default_profile = config::Profile::default();
    let selected_profile = app
        .selected_profile_name()
        .and_then(|name| {
            app.config
                .projects
                .get(&app.path)
                .and_then(|pc| pc.profiles.get(name))
        })
        .unwrap_or(&default_profile);

    // Render Settings Header
    let size_str = selected_profile
        .max_file_size
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Default (10MB)".to_string());
    let depth_str = selected_profile
        .depth
        .map(|d| d.to_string())
        .unwrap_or_else(|| "Unlimited".to_string());

    let settings_text = format!(" Max Size: {} bytes | Depth: {} ", size_str, depth_str);
    let settings_header = Paragraph::new(settings_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Settings ")
                .border_style(Style::default().fg(if is_edit_focused {
                    Color::Cyan
                } else {
                    Color::DarkGray
                })),
        )
        .alignment(Alignment::Left);
    f.render_widget(settings_header, right_pane_chunks[0]);

    // Render Lists
    let render_edit_list = |f: &mut Frame,
                            area: Rect,
                            title: &str,
                            items: &[String],
                            is_active: bool,
                            state: &mut ratatui::widgets::ListState| {
        let list_items: Vec<ListItem> = items.iter().map(|i| ListItem::new(i.clone())).collect();
        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(if is_edit_focused && is_active {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    })),
            )
            .highlight_style(
                Style::default()
                    .bg(if is_edit_focused && is_active {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    })
                    .fg(Color::Black),
            )
            .highlight_symbol(if is_active { "> " } else { "  " });
        f.render_stateful_widget(list, area, state);
    };

    let mut empty_state1 = ratatui::widgets::ListState::default();
    let mut empty_state2 = ratatui::widgets::ListState::default();
    let mut empty_state3 = ratatui::widgets::ListState::default();

    render_edit_list(
        f,
        edit_chunks[0],
        " Exclude ",
        &selected_profile.exclude,
        app.edit_list == EditList::Exclude,
        if app.edit_list == EditList::Exclude {
            &mut app.edit_state
        } else {
            &mut empty_state1
        },
    );
    render_edit_list(
        f,
        edit_chunks[1],
        " Include Only ",
        &selected_profile.include_only,
        app.edit_list == EditList::IncludeOnly,
        if app.edit_list == EditList::IncludeOnly {
            &mut app.edit_state
        } else {
            &mut empty_state2
        },
    );
    render_edit_list(
        f,
        edit_chunks[2],
        " Include Hidden ",
        &selected_profile.include_hidden,
        app.edit_list == EditList::IncludeHidden,
        if app.edit_list == EditList::IncludeHidden {
            &mut app.edit_state
        } else {
            &mut empty_state3
        },
    );

    // Footer
    let footer_text = match app.mode {
        AppMode::ProfileList => {
            "[↑/↓] Nav  [n] New  [r] Rename  [c] Clone  [d] Del  [b] Bind  [u] Unbind  [→/e] Edit  [q] Quit"
        }
        AppMode::EditProfile => {
            "[↑/↓] Sel  [Tab] Switch  [s] Size  [p] Depth  [a] Add  [e] Edit  [x] Del  [←/Esc] Back"
        }
        AppMode::Input => "[Enter] Save  [Esc] Cancel",
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, main_chunks[2]);

    // Input Popup
    if app.mode == AppMode::Input {
        let area = centered_rect(60, 20, size);
        let title = match &app.input_purpose {
            Some(InputPurpose::NewProfile) => " New Profile Name ",
            Some(InputPurpose::RenameProfile(_)) => " Rename Profile ",
            Some(InputPurpose::DuplicateProfile(_)) => " Clone Profile ",
            Some(InputPurpose::AddExclude) => " Add Exclude ",
            Some(InputPurpose::AddIncludeOnly) => " Add Include Only ",
            Some(InputPurpose::AddIncludeHidden) => " Add Include Hidden ",
            Some(InputPurpose::EditExclude(_)) => " Edit Exclude ",
            Some(InputPurpose::EditIncludeOnly(_)) => " Edit Include Only ",
            Some(InputPurpose::EditIncludeHidden(_)) => " Edit Include Hidden ",
            Some(InputPurpose::EditMaxSize) => " Max Size in bytes (Empty for default) ",
            Some(InputPurpose::EditDepth) => " Max Depth (Empty for unlimited) ",
            None => " Input ",
        };

        let input_text = format!("{}█", app.input);
        let input_block = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(Clear, area);
        f.render_widget(input_block, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
