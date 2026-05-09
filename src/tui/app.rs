use crate::config::{self, Config};
use std::path::Path;

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

#[derive(Clone)]
pub enum InputPurpose {
    NewProfile,
    RenameProfile(String),
    DuplicateProfile(String),
    AddExclude,
    AddIncludeOnly,
    AddIncludeHidden,
    EditExclude(usize),
    EditIncludeOnly(usize),
    EditIncludeHidden(usize),
    EditMaxSize,
    EditDepth,
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
    pub fn new(path: String) -> anyhow::Result<Self> {
        let mut app = Self {
            config: config::load_config()?,
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
        Ok(app)
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
        let _ = config::save_config(&self.config);
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
    pub fn current_edit_list_len(&self) -> usize {
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

        let purpose = if let Some(p) = self.input_purpose.clone() {
            p
        } else {
            return;
        };

        // For sizing configs, we allow empty value (resets to None). Other strings we abort.
        if val.is_empty() && !matches!(purpose, InputPurpose::EditMaxSize | InputPurpose::EditDepth)
        {
            self.cancel_input();
            return;
        }

        match purpose {
            InputPurpose::NewProfile => {
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
            InputPurpose::RenameProfile(old_name) => {
                let proj = self.config.projects.entry(self.path.clone()).or_default();
                if !proj.profiles.contains_key(&val) && old_name != val {
                    if let Some(prof) = proj.profiles.remove(&old_name) {
                        proj.profiles.insert(val.clone(), prof);
                        if proj.bound_profile.as_ref() == Some(&old_name) {
                            proj.bound_profile = Some(val.clone());
                        }
                        self.save();
                        self.sync_profiles();
                        if let Some(pos) = self.profiles.iter().position(|p| p == &val) {
                            self.profile_state.select(Some(pos));
                        }
                    }
                }
                self.mode = AppMode::ProfileList;
            }
            InputPurpose::DuplicateProfile(src_name) => {
                let proj = self.config.projects.entry(self.path.clone()).or_default();
                if !proj.profiles.contains_key(&val) {
                    if let Some(prof) = proj.profiles.get(&src_name).cloned() {
                        proj.profiles.insert(val.clone(), prof);
                        self.save();
                        self.sync_profiles();
                        if let Some(pos) = self.profiles.iter().position(|p| p == &val) {
                            self.profile_state.select(Some(pos));
                        }
                    }
                }
                self.mode = AppMode::ProfileList;
            }
            InputPurpose::EditMaxSize => {
                if let Some(prof_name) = self.selected_profile_name().cloned() {
                    if let Some(proj) = self.config.projects.get_mut(&self.path) {
                        if let Some(prof) = proj.profiles.get_mut(&prof_name) {
                            prof.max_file_size = if val.is_empty() {
                                None
                            } else {
                                val.parse().ok()
                            };
                            self.save();
                        }
                    }
                }
                self.mode = AppMode::EditProfile;
            }
            InputPurpose::EditDepth => {
                if let Some(prof_name) = self.selected_profile_name().cloned() {
                    if let Some(proj) = self.config.projects.get_mut(&self.path) {
                        if let Some(prof) = proj.profiles.get_mut(&prof_name) {
                            prof.depth = if val.is_empty() {
                                None
                            } else {
                                val.parse().ok()
                            };
                            self.save();
                        }
                    }
                }
                self.mode = AppMode::EditProfile;
            }
            _ => {
                let mut mutated = false;
                let mut new_idx = None;

                if let Some(prof_name) = self.selected_profile_name().cloned() {
                    if let Some(proj) = self.config.projects.get_mut(&self.path) {
                        if let Some(prof) = proj.profiles.get_mut(&prof_name) {
                            let list = match purpose {
                                InputPurpose::AddExclude | InputPurpose::EditExclude(_) => {
                                    &mut prof.exclude
                                }
                                InputPurpose::AddIncludeOnly | InputPurpose::EditIncludeOnly(_) => {
                                    &mut prof.include_only
                                }
                                InputPurpose::AddIncludeHidden
                                | InputPurpose::EditIncludeHidden(_) => &mut prof.include_hidden,
                                _ => unreachable!(),
                            };

                            if let InputPurpose::EditExclude(idx)
                            | InputPurpose::EditIncludeOnly(idx)
                            | InputPurpose::EditIncludeHidden(idx) = purpose
                            {
                                if idx < list.len() && list[idx] != val {
                                    let old = list.remove(idx);
                                    if !list.contains(&val) {
                                        list.push(val.clone());
                                        mutated = true;
                                    } else {
                                        list.push(old); // duplicate exists, revert
                                    }
                                    list.sort();
                                    new_idx = list.iter().position(|x| x == &val);
                                }
                            } else {
                                // Add logic
                                if !list.contains(&val) {
                                    list.push(val.clone());
                                    list.sort();
                                    mutated = true;
                                }
                                new_idx = list.iter().position(|x| x == &val);
                            }
                        }
                    }
                }

                if mutated {
                    self.save();
                }
                if let Some(idx) = new_idx {
                    self.edit_state.select(Some(idx));
                }
                self.mode = AppMode::EditProfile;
            }
        }
        self.input.clear();
        self.input_purpose = None;
    }

    pub fn cancel_input(&mut self) {
        self.mode = match &self.input_purpose {
            Some(InputPurpose::NewProfile)
            | Some(InputPurpose::RenameProfile(_))
            | Some(InputPurpose::DuplicateProfile(_)) => AppMode::ProfileList,
            _ => AppMode::EditProfile,
        };
        self.input.clear();
        self.input_purpose = None;
    }
}
