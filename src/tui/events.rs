use super::app::{App, AppMode, EditList, InputPurpose};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_events(key: KeyEvent, app: &mut App) {
    match app.mode {
        AppMode::ProfileList => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Down | KeyCode::Char('j') => app.next_profile(),
            KeyCode::Up | KeyCode::Char('k') => app.prev_profile(),
            KeyCode::Char('n') => {
                app.mode = AppMode::Input;
                app.input_purpose = Some(InputPurpose::NewProfile);
            }
            KeyCode::Char('r') => {
                if let Some(name) = app.selected_profile_name().cloned() {
                    app.mode = AppMode::Input;
                    app.input = name.clone();
                    app.input_purpose = Some(InputPurpose::RenameProfile(name));
                }
            }
            KeyCode::Char('c') => {
                if let Some(name) = app.selected_profile_name().cloned() {
                    app.mode = AppMode::Input;
                    app.input = format!("{}_copy", name);
                    app.input_purpose = Some(InputPurpose::DuplicateProfile(name));
                }
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
            KeyCode::Char('s') => {
                app.mode = AppMode::Input;
                app.input_purpose = Some(InputPurpose::EditMaxSize);
                if let Some(p) = app.selected_profile_name().cloned() {
                    if let Some(prof) = app
                        .config
                        .projects
                        .get(&app.path)
                        .and_then(|pc| pc.profiles.get(&p))
                    {
                        app.input = prof
                            .max_file_size
                            .map(|s| s.to_string())
                            .unwrap_or_default();
                    }
                }
            }
            KeyCode::Char('p') => {
                app.mode = AppMode::Input;
                app.input_purpose = Some(InputPurpose::EditDepth);
                if let Some(p) = app.selected_profile_name().cloned() {
                    if let Some(prof) = app
                        .config
                        .projects
                        .get(&app.path)
                        .and_then(|pc| pc.profiles.get(&p))
                    {
                        app.input = prof.depth.map(|d| d.to_string()).unwrap_or_default();
                    }
                }
            }
            KeyCode::Char('a') => {
                app.mode = AppMode::Input;
                app.input_purpose = match app.edit_list {
                    EditList::Exclude => Some(InputPurpose::AddExclude),
                    EditList::IncludeOnly => Some(InputPurpose::AddIncludeOnly),
                    EditList::IncludeHidden => Some(InputPurpose::AddIncludeHidden),
                };
            }
            KeyCode::Enter | KeyCode::Char('e') => {
                if let Some(prof_name) = app.selected_profile_name().cloned() {
                    if let Some(idx) = app.edit_state.selected() {
                        if let Some(p) = app
                            .config
                            .projects
                            .get(&app.path)
                            .and_then(|pc| pc.profiles.get(&prof_name))
                        {
                            let (val, purpose) = match app.edit_list {
                                EditList::Exclude => {
                                    (p.exclude.get(idx).cloned(), InputPurpose::EditExclude(idx))
                                }
                                EditList::IncludeOnly => (
                                    p.include_only.get(idx).cloned(),
                                    InputPurpose::EditIncludeOnly(idx),
                                ),
                                EditList::IncludeHidden => (
                                    p.include_hidden.get(idx).cloned(),
                                    InputPurpose::EditIncludeHidden(idx),
                                ),
                            };
                            if let Some(v) = val {
                                app.mode = AppMode::Input;
                                app.input = v;
                                app.input_purpose = Some(purpose);
                            }
                        }
                    }
                }
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
