use crate::config::{self, Config};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend}, layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
    Terminal,
};
use std::{io, path::Path, time::Duration};

#[derive(PartialEq, Debug)]
pub enum AppMode {
    ProfileList,
    EditProfile,
    Input,
}

#[derive(PartialEq, Clone, Copy)]
pub enum EditList {
    Exclude,
    IncludeOnly,
    IncludeHidden,
}

#[derive(Clone, Copy)]
pub enum InputPurpose {
    NewProfile,
    AddExclude,
    AddIncludeOnly,
    AddIncludeHidden,
}

pub struct App {
    pub config: Config,
    pub path: String,
    pub mode: AppMode,

    pub profiles: Vec<String>,
    pub profile_state: ratatui::widgets::ListState,

    pub edit_list: EditList,
    pub edit_state: ratatui::widgets::ListState,

    pub input: String,
    pub input_purpose: Option<InputPurpose>,

    pub should_quit: bool,
}

impl App {
    pub fn new(path: String) -> Self {
        let mut app = Self {
            config: config::load_config(),
            path: dunce::canonicalize(Path::new(&path))
                .unwrap_or_else(|_| Path::new(&path).to_path_buf())
                .to_string_lossy()
                .to_string(),
            mode: AppMode::ProfileList,
            profiles: Vec::new(),
            profile_state: ratatui::widgets::ListState::default(),
            edit_list: EditList::Exclude,
            edit_state: ratatui::widgets::ListState::default(),
            input: String::new(),
            input_purpose: None,
            should_quit: false,
        };
        app.sync_profiles();
        if !app.profiles.is_empty() {
            app.profile_state.select(Some(0));
        }
        app
    }

    pub fn sync_profiles(&mut self) {
        self.profiles = self
            .config
            .projects
            .get(&self.path)
            .map(|p| p.profiles.keys().cloned().collect())
            .unwrap_or_default();
        self.profiles.sort();
    }

    pub fn selected_profile_name(&self) -> Option<&String> {
        self.profile_state
            .selected()
            .and_then(|i| self.profiles.get(i))
    }

    pub fn bound_profile_name(&self) -> Option<&String> {
        self.config
            .projects
            .get(&self.path)
            .and_then(|p| p.bound_profile.as_ref())
    }

    pub fn save(&self) {
        config::save_config(&self.config);
    }

    // --- Profile Navigation ---
    pub fn next_profile(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        let i = match self.profile_state.selected() {
            Some(i) => {
                if i >= self.profiles.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.profile_state.select(Some(i));
    }

    pub fn prev_profile(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        let i = match self.profile_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.profiles.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.profile_state.select(Some(i));
    }

    // --- Edit Navigation ---
    fn current_edit_list_len(&self) -> usize {
        let name = match self.selected_profile_name() {
            Some(n) => n,
            None => return 0,
        };
        let prof = match self
            .config
            .projects
            .get(&self.path)
            .and_then(|pc| pc.profiles.get(name))
        {
            Some(p) => p,
            None => return 0,
        };
        match self.edit_list {
            EditList::Exclude => prof.exclude.len(),
            EditList::IncludeOnly => prof.include_only.len(),
            EditList::IncludeHidden => prof.include_hidden.len(),
        }
    }

    pub fn next_edit_item(&mut self) {
        let len = self.current_edit_list_len();
        if len == 0 {
            self.edit_state.select(None);
            return;
        }
        let i = match self.edit_state.selected() {
            Some(i) => {
                if i >= len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.edit_state.select(Some(i));
    }

    pub fn prev_edit_item(&mut self) {
        let len = self.current_edit_list_len();
        if len == 0 {
            self.edit_state.select(None);
            return;
        }
        let i = match self.edit_state.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.edit_state.select(Some(i));
    }

    pub fn cycle_edit_list(&mut self, forward: bool) {
        self.edit_list = match (self.edit_list, forward) {
            (EditList::Exclude, true) => EditList::IncludeOnly,
            (EditList::Exclude, false) => EditList::IncludeHidden,
            (EditList::IncludeOnly, true) => EditList::IncludeHidden,
            (EditList::IncludeOnly, false) => EditList::Exclude,
            (EditList::IncludeHidden, true) => EditList::Exclude,
            (EditList::IncludeHidden, false) => EditList::IncludeOnly,
        };
        let len = self.current_edit_list_len();
        self.edit_state.select(if len > 0 { Some(0) } else { None });
    }

    // --- Actions ---
    pub fn delete_selected_profile(&mut self) {
        if let Some(name) = self.selected_profile_name().cloned() {
            if let Some(proj) = self.config.projects.get_mut(&self.path) {
                proj.profiles.remove(&name);
                if proj.bound_profile.as_ref() == Some(&name) {
                    proj.bound_profile = None;
                }
            }
            self.save();
            self.sync_profiles();
            if self.profiles.is_empty() {
                self.profile_state.select(None);
            } else {
                let current = self.profile_state.selected().unwrap_or(0);
                self.profile_state
                    .select(Some(current.min(self.profiles.len() - 1)));
            }
        }
    }

    pub fn delete_selected_edit_item(&mut self) {
        let prof_name = match self.selected_profile_name().cloned() {
            Some(n) => n,
            None => return,
        };
        let idx = match self.edit_state.selected() {
            Some(i) => i,
            None => return,
        };

        let mut mutated = false;
        let mut list_is_empty = false;
        let mut new_len = 0;

        if let Some(proj) = self.config.projects.get_mut(&self.path) {
            if let Some(prof) = proj.profiles.get_mut(&prof_name) {
                let list = match self.edit_list {
                    EditList::Exclude => &mut prof.exclude,
                    EditList::IncludeOnly => &mut prof.include_only,
                    EditList::IncludeHidden => &mut prof.include_hidden,
                };
                if idx < list.len() {
                    list.remove(idx);
                    mutated = true;
                    list_is_empty = list.is_empty();
                    new_len = list.len();
                }
            }
        }

        if mutated {
            self.save();
            if list_is_empty {
                self.edit_state.select(None);
            } else {
                self.edit_state.select(Some(idx.min(new_len - 1)));
            }
        }
    }

    pub fn submit_input(&mut self) {
        let val = self.input.trim().to_string();
        if val.is_empty() {
            self.cancel_input();
            return;
        }

        match self.input_purpose {
            Some(InputPurpose::NewProfile) => {
                let proj = self.config.projects.entry(self.path.clone()).or_default();
                if !proj.profiles.contains_key(&val) {
                    proj.profiles
                        .insert(val.clone(), config::Profile::default());
                    self.save();
                    self.sync_profiles();
                    if let Some(pos) = self.profiles.iter().position(|p| p == &val) {
                        self.profile_state.select(Some(pos));
                    }
                }
                self.mode = AppMode::ProfileList;
            }
            Some(purpose) => {
                let mut mutated = false;
                let mut new_len = 0;

                if let Some(prof_name) = self.selected_profile_name().cloned() {
                    if let Some(proj) = self.config.projects.get_mut(&self.path) {
                        if let Some(prof) = proj.profiles.get_mut(&prof_name) {
                            let list = match purpose {
                                InputPurpose::AddExclude => &mut prof.exclude,
                                InputPurpose::AddIncludeOnly => &mut prof.include_only,
                                InputPurpose::AddIncludeHidden => &mut prof.include_hidden,
                                _ => unreachable!(),
                            };
                            if !list.contains(&val) {
                                list.push(val);
                                list.sort();
                                mutated = true;
                            }
                            new_len = list.len();
                        }
                    }
                }

                if mutated {
                    self.save();
                }
                self.edit_state.select(Some(new_len.saturating_sub(1)));
                self.mode = AppMode::EditProfile;
            }
            None => {}
        }
        self.input.clear();
        self.input_purpose = None;
    }

    pub fn cancel_input(&mut self) {
        self.mode = match self.input_purpose {
            Some(InputPurpose::NewProfile) => AppMode::ProfileList,
            _ => AppMode::EditProfile,
        };
        self.input.clear();
        self.input_purpose = None;
    }
}

pub fn run(path: String) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(path);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui(f, app)).expect("TUI draw failed");

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_events(key, app);
                }
            }
        }
    }
    Ok(())
}

fn handle_key_events(key: KeyEvent, app: &mut App) {
    match app.mode {
        AppMode::ProfileList => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => app.next_profile(),
            KeyCode::Up | KeyCode::Char('k') => app.prev_profile(),
            KeyCode::Char('n') => {
                app.mode = AppMode::Input;
                app.input_purpose = Some(InputPurpose::NewProfile);
            }
            KeyCode::Char('d') => app.delete_selected_profile(),
            KeyCode::Char('b') => {
                if let Some(name) = app.selected_profile_name().cloned() {
                    let proj = app.config.projects.entry(app.path.clone()).or_default();
                    proj.bound_profile = Some(name);
                    app.save();
                }
            }
            KeyCode::Char('u') => {
                if let Some(proj) = app.config.projects.get_mut(&app.path) {
                    proj.bound_profile = None;
                    app.save();
                }
            }
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('e') => {
                if !app.profiles.is_empty() {
                    app.mode = AppMode::EditProfile;
                    app.edit_list = EditList::Exclude;
                    let len = app.current_edit_list_len();
                    app.edit_state.select(if len > 0 { Some(0) } else { None });
                }
            }
            _ => {}
        },
        AppMode::EditProfile => match key.code {
            KeyCode::Left | KeyCode::Esc => app.mode = AppMode::ProfileList,
            KeyCode::Down | KeyCode::Char('j') => app.next_edit_item(),
            KeyCode::Up | KeyCode::Char('k') => app.prev_edit_item(),
            KeyCode::Tab => app.cycle_edit_list(true),
            KeyCode::BackTab => app.cycle_edit_list(false),
            KeyCode::Char('a') => {
                app.mode = AppMode::Input;
                app.input_purpose = match app.edit_list {
                    EditList::Exclude => Some(InputPurpose::AddExclude),
                    EditList::IncludeOnly => Some(InputPurpose::AddIncludeOnly),
                    EditList::IncludeHidden => Some(InputPurpose::AddIncludeHidden),
                };
            }
            KeyCode::Char('x') => app.delete_selected_edit_item(),
            _ => {}
        },
        AppMode::Input => match key.code {
            KeyCode::Enter => app.submit_input(),
            KeyCode::Esc => app.cancel_input(),
            KeyCode::Backspace => {
                app.input.pop();
            }
            KeyCode::Char(c) => {
                app.input.push(c);
            }
            _ => {}
        },
    }
}

fn ui(f: &mut Frame, app: &mut App) {
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
    let edit_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(body_chunks[1]);

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
            "[↑/↓] Navigate  [n] New  [d] Delete  [b] Bind  [u] Unbind  [→/e] Edit  [q] Quit"
        }
        AppMode::EditProfile => {
            "[↑/↓] Select  [Tab] Switch List  [a] Add  [x] Delete  [←/Esc] Back"
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
        let title = match app.input_purpose {
            Some(InputPurpose::NewProfile) => " New Profile Name ",
            Some(InputPurpose::AddExclude) => " Add Exclude ",
            Some(InputPurpose::AddIncludeOnly) => " Add Include Only ",
            Some(InputPurpose::AddIncludeHidden) => " Add Include Hidden ",
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
