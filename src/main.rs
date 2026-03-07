mod file_entry;
mod theme;
mod transfer;

use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use file_entry::{sort_entries, FileEntry, SortColumn};
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, svg, text, text_input, Column,
    Row, Space,
};
use iced::{alignment, mouse, Alignment, Background, Color, Element, Event, Font, Length, Padding, Point, Size, Subscription, Task, Theme};
use sysinfo::System;
use std::path::PathBuf;
use std::time::Instant;

// ─── Font ────────────────────────────────────────────────────────────────────
const JETBRAINS_MONO_BYTES: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");

// ─── Icons ───────────────────────────────────────────────────────────────────
const ICON_FOLDER: &[u8] = include_bytes!("../assets/icons/folder.svg");
const ICON_FILE: &[u8] = include_bytes!("../assets/icons/file.svg");
const ICON_EXEC: &[u8] = include_bytes!("../assets/icons/executable.svg");
const ICON_PARENT: &[u8] = include_bytes!("../assets/icons/parent.svg");
const ICON_DRIVE_LOCAL: &[u8] = include_bytes!("../assets/icons/local.svg");
const ICON_PROJECTS: &[u8] = include_bytes!("../assets/icons/projects.svg");
const ICON_DOWNLOADS: &[u8] = include_bytes!("../assets/icons/downloads.svg");
const ICON_PICTURES: &[u8] = include_bytes!("../assets/icons/pictures.svg");
const ICON_DOCUMENTS: &[u8] = include_bytes!("../assets/icons/documents.svg");
const ICON_VIDEOS: &[u8] = include_bytes!("../assets/icons/videos.svg");
const ICON_OPEN: &[u8] = include_bytes!("../assets/icons/open.svg");
const ICON_OPEN_WITH: &[u8] = include_bytes!("../assets/icons/open_with.svg");
const ICON_RENAME: &[u8] = include_bytes!("../assets/icons/rename.svg");
const ICON_COPY: &[u8] = include_bytes!("../assets/icons/copy.svg");
const ICON_CUT: &[u8] = include_bytes!("../assets/icons/cut.svg");
const ICON_DELETE: &[u8] = include_bytes!("../assets/icons/delete.svg");
const ICON_PROPERTIES: &[u8] = include_bytes!("../assets/icons/properties.svg");
const ICON_PASTE: &[u8] = include_bytes!("../assets/icons/paste.svg");
const ICON_ORBITAL: &[u8] = include_bytes!("../assets/icons/Orbital.svg");
const ICON_NEW_FOLDER: &[u8] = include_bytes!("../assets/icons/new_folder.svg");
const ICON_REFRESH: &[u8] = include_bytes!("../assets/icons/refresh.svg");
const ICON_SORT: &[u8] = include_bytes!("../assets/icons/sort.svg");
const ICON_SELECT_ALL: &[u8] = include_bytes!("../assets/icons/select_all.svg");
const ICON_TERMINAL: &[u8] = include_bytes!("../assets/icons/terminal.svg");

fn mono_font() -> Font {
    Font::with_name("JetBrains Mono")
}

// ─── Enums ───────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActiveMenu {
    None,
    File,
    Edit,
    View,
    Sort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    List,
    Grid,
    Compact,
}

// ─── Application State ──────────────────────────────────────────────────────
struct OrbitalHud {
    current_path: PathBuf,
    entries: Vec<FileEntry>,
    sort_column: SortColumn,
    sort_ascending: bool,
    search_query: String,
    cpu_usage: f32,
    mem_usage: f32,
    selected_entry: Option<usize>,
    renaming: bool,
    rename_input: String,
    last_click_time: Option<Instant>,
    last_click_idx: Option<usize>,
    active_menu: ActiveMenu,
    view_mode: ViewMode,
    show_hidden: bool,
    show_sidebar: bool,
    dragging_item: Option<usize>,
    drag_start_pos: Option<Point>,
    current_drag_pos: Option<Point>,
    hovered_row: Option<usize>,
    current_raw_mouse_pos: Option<Point>,
    context_menu: Option<(Option<usize>, Point)>,
    clipboard: Option<(PathBuf, bool)>, // (path, is_cut)
    show_properties: Option<usize>,
    active_transfer: Option<Arc<Mutex<transfer::TransferInfo>>>,
    transfer_abort_flag: Option<Arc<AtomicBool>>,
    error_notification: Option<(String, Instant)>,
    clipboard_notification: Option<(String, Instant)>,
}

// ─── Messages ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
enum Message {
    NavigateTo(PathBuf),
    NavigateUp,
    EntryClicked(usize),
    SortBy(SortColumn),
    SearchChanged(String),
    BreadcrumbClicked(usize),
    // Menu toggles
    ToggleMenu(ActiveMenu),
    CloseMenus,
    // File menu actions
    NewFolder,
    StartRename,
    RenameInputChanged(String),
    ConfirmRename,
    AbortRename,
    DuplicateEntry,
    MoveToTrash,
    // View menu actions
    ToggleHidden,
    RefreshDir,
    ToggleSidebar,
    // Sort menu actions
    SetSortAscending,
    SetSortDescending,
    Noop,
    // Drag and Drop
    HoverRow(usize),
    LeaveRow(usize),
    GlobalEvent(Event),
    // Context Menu actions
    ContextMenuAction(String, Option<usize>), // (action identifier, item index option)
    PasteClipboard(Option<PathBuf>),
    // Clipboard ops from keyboard/edit menu
    CopySelected,
    CutSelected,
    SelectAll,
    OpenInTerminal(Option<PathBuf>),
    // Transfer Engine
    TickTransfer(Instant),
    AbortTransfer,
    // Notifications
    DismissNotification,
}

impl OrbitalHud {
    fn new() -> (Self, Task<Message>) {
        let home = dirs_or_root();
        let entries = FileEntry::read_directory(&home);

        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let cpu = sys.global_cpu_usage();
        let mem_total = sys.total_memory() as f64;
        let mem_used = sys.used_memory() as f64;
        let mem_pct = if mem_total > 0.0 {
            (mem_used / mem_total * 100.0) as f32
        } else {
            0.0
        };

        (
            Self {
                current_path: home,
                entries,
                sort_column: SortColumn::Name,
                sort_ascending: true,
                search_query: String::new(),
                cpu_usage: cpu,
                mem_usage: mem_pct,
                selected_entry: None,
                renaming: false,
                rename_input: String::new(),
                last_click_time: None,
                last_click_idx: None,
                active_menu: ActiveMenu::None,
                view_mode: ViewMode::List,
                show_hidden: false,
                show_sidebar: true,
                dragging_item: None,
                drag_start_pos: None,
                current_drag_pos: None,
                hovered_row: None,
                current_raw_mouse_pos: None,
                context_menu: None,
                clipboard: None,
                show_properties: None,
                active_transfer: None,
                transfer_abort_flag: None,
                error_notification: None,
                clipboard_notification: None,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        format!("Orbital HUD — {}", self.current_path.display())
    }

    fn refresh_entries(&mut self) {
        self.entries = FileEntry::read_directory(&self.current_path);
        if !self.show_hidden {
            self.entries.retain(|e| e.is_parent || !e.name.starts_with('.'));
        }
        sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(path) => {
                if path.is_dir() {
                    let canonical = path.canonicalize().unwrap_or(path);
                    self.current_path = canonical;
                    self.refresh_entries();
                    self.selected_entry = None;
                    self.renaming = false;
                }
            }
            Message::NavigateUp => {
                if let Some(parent) = self.current_path.parent() {
                    let parent = parent.to_path_buf();
                    self.current_path = parent;
                    self.refresh_entries();
                    self.selected_entry = None;
                    self.renaming = false;
                }
            }
            Message::EntryClicked(idx) => {
                self.active_menu = ActiveMenu::None;
                if self.renaming && self.selected_entry == Some(idx) {
                    return Task::none();
                }
                if self.renaming {
                    self.renaming = false;
                }
                
                // If we were dragging and just released over the same item...
                // Handled in GlobalEvent, but we keep this standard click logic.
                
                let now = Instant::now();
                let is_double = match (self.last_click_idx, self.last_click_time) {
                    (Some(li), Some(lt)) => li == idx && now.duration_since(lt).as_millis() < 400,
                    _ => false,
                };
                self.last_click_time = Some(now);
                self.last_click_idx = Some(idx);

                if is_double {
                    self.selected_entry = None;
                    self.last_click_time = None;
                    self.last_click_idx = None;
                    if let Some(entry) = self.entries.get(idx) {
                        if entry.is_parent {
                            return self.update(Message::NavigateUp);
                        } else if entry.is_dir {
                            let dir_name = entry.name.trim_end_matches('/');
                            let new_path = self.current_path.join(dir_name);
                            return self.update(Message::NavigateTo(new_path));
                        }
                    }
                } else {
                    self.selected_entry = Some(idx);
                }
            }
            Message::SortBy(col) => {
                self.active_menu = ActiveMenu::None;
                if self.sort_column == col {
                    self.sort_ascending = !self.sort_ascending;
                } else {
                    self.sort_column = col;
                    self.sort_ascending = true;
                }
                sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
            }
            Message::SearchChanged(q) => {
                self.search_query = q;
            }
            Message::BreadcrumbClicked(segment_idx) => {
                let components: Vec<_> = self.current_path.components().collect();
                if segment_idx < components.len() {
                    let mut target = PathBuf::new();
                    for comp in components.iter().take(segment_idx + 1) {
                        target.push(comp);
                    }
                    return self.update(Message::NavigateTo(target));
                }
            }
            Message::ToggleMenu(menu) => {
                self.active_menu = if self.active_menu == menu {
                    ActiveMenu::None
                } else {
                    menu
                };
            }
            Message::CloseMenus => {
                self.active_menu = ActiveMenu::None;
                self.show_properties = None;
                self.context_menu = None;
                if self.renaming {
                    self.renaming = false;
                }
            }
            Message::NewFolder => {
                self.active_menu = ActiveMenu::None;
                let mut new_name = "New Folder".to_string();
                let mut counter = 1;
                while self.current_path.join(&new_name).exists() {
                    new_name = format!("New Folder ({})", counter);
                    counter += 1;
                }
                let _ = std::fs::create_dir(self.current_path.join(&new_name));
                self.refresh_entries();
                if let Some(idx) = self.entries.iter().position(|e| {
                    e.is_dir && e.name.trim_end_matches('/') == new_name
                }) {
                    self.selected_entry = Some(idx);
                    self.rename_input = new_name;
                    self.renaming = true;
                    return text_input::focus(text_input::Id::new("rename_input"));
                }
            }
            Message::StartRename => {
                self.active_menu = ActiveMenu::None;
                if !self.renaming {
                    if let Some(idx) = self.selected_entry {
                        if let Some(entry) = self.entries.get(idx) {
                            if !entry.is_parent {
                                self.rename_input = if entry.is_dir {
                                    entry.name.trim_end_matches('/').to_string()
                                } else {
                                    entry.name.clone()
                                };
                                self.renaming = true;
                                return text_input::focus(text_input::Id::new("rename_input"));
                            }
                        }
                    }
                }
            }
            Message::RenameInputChanged(val) => {
                self.rename_input = val;
            }
            Message::ConfirmRename => {
                if self.renaming {
                    if let Some(idx) = self.selected_entry {
                        if let Some(entry) = self.entries.get(idx) {
                            let old_name = if entry.is_dir {
                                entry.name.trim_end_matches('/').to_string()
                            } else {
                                entry.name.clone()
                            };
                            let new_name = self.rename_input.trim().to_string();
                            if !new_name.is_empty() && new_name != old_name {
                                let old_path = self.current_path.join(&old_name);
                                let new_path = self.current_path.join(&new_name);
                                let _ = std::fs::rename(&old_path, &new_path);
                            }
                        }
                    }
                    self.renaming = false;
                    self.selected_entry = None;
                    self.refresh_entries();
                }
            }
            Message::AbortRename => {
                self.renaming = false;
                self.active_menu = ActiveMenu::None;
            }
            Message::DuplicateEntry => {
                self.active_menu = ActiveMenu::None;
                if let Some(idx) = self.selected_entry {
                    if let Some(entry) = self.entries.get(idx) {
                        if !entry.is_parent {
                            let old_name = if entry.is_dir {
                                entry.name.trim_end_matches('/').to_string()
                            } else {
                                entry.name.clone()
                            };
                            let old_path = self.current_path.join(&old_name);
                            let mut new_name = format!("{} (copy)", old_name);
                            let mut counter = 2;
                            while self.current_path.join(&new_name).exists() {
                                new_name = format!("{} (copy {})", old_name, counter);
                                counter += 1;
                            }
                            let new_path = self.current_path.join(&new_name);
                            if entry.is_dir {
                                let _ = std::fs::create_dir(&new_path);
                            } else {
                                let _ = std::fs::copy(&old_path, &new_path);
                            }
                            self.refresh_entries();
                        }
                    }
                }
            }
            Message::MoveToTrash => {
                self.active_menu = ActiveMenu::None;
                if let Some(idx) = self.selected_entry {
                    if let Some(entry) = self.entries.get(idx) {
                        if !entry.is_parent {
                            let name = if entry.is_dir {
                                entry.name.trim_end_matches('/').to_string()
                            } else {
                                entry.name.clone()
                            };
                            let path = self.current_path.join(&name);
                            if entry.is_dir {
                                let _ = std::fs::remove_dir_all(&path);
                            } else {
                                let _ = std::fs::remove_file(&path);
                            }
                            self.selected_entry = None;
                            self.refresh_entries();
                        }
                    }
                }
            }
            Message::ToggleHidden => {
                self.active_menu = ActiveMenu::None;
                self.show_hidden = !self.show_hidden;
                self.refresh_entries();
            }
            Message::RefreshDir => {
                self.active_menu = ActiveMenu::None;
                self.refresh_entries();
            }
            Message::ToggleSidebar => {
                self.active_menu = ActiveMenu::None;
                self.show_sidebar = !self.show_sidebar;
            }
            Message::SetSortAscending => {
                self.active_menu = ActiveMenu::None;
                self.sort_ascending = true;
                sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
            }
            Message::SetSortDescending => {
                self.active_menu = ActiveMenu::None;
                self.sort_ascending = false;
                sort_entries(&mut self.entries, self.sort_column, self.sort_ascending);
            }
            Message::Noop => { self.active_menu = ActiveMenu::None; }
            Message::HoverRow(idx) => {
                self.hovered_row = Some(idx);
            }
            Message::LeaveRow(idx) => {
                if self.hovered_row == Some(idx) {
                    self.hovered_row = None;
                }
            }
            Message::GlobalEvent(event) => {
                match event {
                    Event::Mouse(mouse::Event::CursorMoved { position }) => {
                        self.current_raw_mouse_pos = Some(position);
                        if self.dragging_item.is_some() {
                            self.current_drag_pos = Some(position);
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                        // Dismiss context menu
                        self.context_menu = None;
                        
                        if let Some(idx) = self.hovered_row {
                            self.dragging_item = Some(idx);
                            self.drag_start_pos = self.current_raw_mouse_pos;
                            self.current_drag_pos = self.current_raw_mouse_pos;
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                        // Open context menu
                        if let Some(pos) = self.current_raw_mouse_pos {
                            self.context_menu = Some((self.hovered_row, pos));
                        }
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        if self.dragging_item.is_some() {
                            // Finish drag
                            if let (Some(drag_idx), Some(target_idx)) = (self.dragging_item, self.hovered_row) {
                                // Calculate distance to ensuring it wasn't just a regular click
                                let moved = match (self.drag_start_pos, self.current_drag_pos) {
                                    (Some(start), Some(curr)) => {
                                        let dx = (start.x - curr.x).abs();
                                        let dy = (start.y - curr.y).abs();
                                        dx > 5.0 || dy > 5.0
                                    }
                                    _ => false,
                                };

                                if drag_idx != target_idx && moved {
                                    let src_entry = &self.entries[drag_idx];
                                    let dst_entry = &self.entries[target_idx];
                                    
                                    // only allow drop into directories or parents
                                    if dst_entry.is_dir || dst_entry.is_parent {
                                        let src_path = self.current_path.join(&src_entry.name);
                                        
                                        // Target can be parent directory, or a regular folder.
                                        let dest_dir = if dst_entry.is_parent {
                                            if let Some(parent) = self.current_path.parent() {
                                                parent.to_path_buf()
                                            } else {
                                                self.current_path.clone()
                                            }
                                        } else {
                                            self.current_path.join(&dst_entry.name)
                                        };
                                        
                                        if let Some(file_name) = src_path.file_name() {
                                            let final_dest = dest_dir.join(file_name);
                                            let _ = std::fs::rename(&src_path, &final_dest);
                                        }
                                    }
                                }
                            }
                            
                            self.dragging_item = None;
                            self.drag_start_pos = None;
                            self.current_drag_pos = None;
                            // Do not clear hovered_row here since the mouse might still be over it
                            
                            self.refresh_entries();
                        }
                    }
                    _ => {}
                }
            }
            Message::ContextMenuAction(action, idx_opt) => {
                self.context_menu = None; // Dismiss menu
                // Validate index is still within bounds (entries may have changed)
                let idx_opt = idx_opt.filter(|&idx| idx < self.entries.len());
                let dest_path = if let Some(idx) = idx_opt {
                    let entry_name = if self.entries[idx].is_dir {
                        self.entries[idx].name.trim_end_matches('/').to_string()
                    } else {
                        self.entries[idx].name.clone()
                    };
                    self.current_path.join(&entry_name)
                } else {
                    self.current_path.clone()
                };

                match action.as_str() {
                    "OPEN" => {
                        if let Some(idx) = idx_opt {
                            if self.entries[idx].is_dir || self.entries[idx].is_parent {
                                return Task::done(Message::NavigateTo(dest_path));
                            } else {
                                // Spawn xdg-open for files
                                match std::process::Command::new("xdg-open")
                                    .arg(&dest_path)
                                    .spawn() {
                                    Ok(_) => {}
                                    Err(e) => {
                                        self.error_notification = Some((
                                            format!("Failed to open file: {}", e),
                                            Instant::now(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    "RENAME" => {
                        if let Some(idx) = idx_opt {
                            if !self.entries[idx].is_parent {
                                self.selected_entry = Some(idx);
                                self.renaming = true;
                                self.rename_input = dest_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                return text_input::focus(text_input::Id::new("rename_input"));
                            }
                        }
                    }
                    "COPY" => {
                        if idx_opt.is_some() {
                            let name = dest_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            self.clipboard = Some((dest_path, false));
                            self.clipboard_notification = Some((
                                format!("Copied: {}", name),
                                Instant::now(),
                            ));
                        }
                    }
                    "CUT" => {
                        if idx_opt.is_some() {
                            let name = dest_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                            self.clipboard = Some((dest_path, true));
                            self.clipboard_notification = Some((
                                format!("Cut: {}", name),
                                Instant::now(),
                            ));
                        }
                    }
                    "DELETE" => {
                        if let Some(idx) = idx_opt {
                            if !self.entries[idx].is_parent {
                                self.selected_entry = Some(idx);
                                return Task::done(Message::MoveToTrash);
                            }
                        }
                    }
                    "PROPERTIES" => {
                        if let Some(idx) = idx_opt {
                            self.show_properties = Some(idx);
                        }
                    }
                    "PASTE" => {
                        // Paste into the target folder or current dir
                        let paste_target = if idx_opt.map_or(false, |idx| self.entries[idx].is_dir || self.entries[idx].is_parent) {
                            dest_path
                        } else {
                            self.current_path.clone()
                        };
                        return Task::done(Message::PasteClipboard(Some(paste_target)));
                    }
                    "NEW_FOLDER" => {
                        return Task::done(Message::NewFolder);
                    }
                    "REFRESH" => {
                        return Task::done(Message::RefreshDir);
                    }
                    "OPEN_TERMINAL" => {
                        let dir = if idx_opt.map_or(false, |idx| self.entries[idx].is_dir || self.entries[idx].is_parent) {
                            dest_path
                        } else {
                            self.current_path.clone()
                        };
                        return Task::done(Message::OpenInTerminal(Some(dir)));
                    }
                    "SELECT_ALL" => {
                        return Task::done(Message::SelectAll);
                    }
                    _ => {}
                }
            }
            Message::CopySelected => {
                self.active_menu = ActiveMenu::None;
                if let Some(idx) = self.selected_entry {
                    if let Some(entry) = self.entries.get(idx) {
                        if !entry.is_parent {
                            let name = if entry.is_dir {
                                entry.name.trim_end_matches('/').to_string()
                            } else {
                                entry.name.clone()
                            };
                            let path = self.current_path.join(&name);
                            self.clipboard = Some((path, false));
                            self.clipboard_notification = Some((
                                format!("Copied: {}", name),
                                Instant::now(),
                            ));
                        }
                    }
                }
            }
            Message::CutSelected => {
                self.active_menu = ActiveMenu::None;
                if let Some(idx) = self.selected_entry {
                    if let Some(entry) = self.entries.get(idx) {
                        if !entry.is_parent {
                            let name = if entry.is_dir {
                                entry.name.trim_end_matches('/').to_string()
                            } else {
                                entry.name.clone()
                            };
                            let path = self.current_path.join(&name);
                            self.clipboard = Some((path, true));
                            self.clipboard_notification = Some((
                                format!("Cut: {}", name),
                                Instant::now(),
                            ));
                        }
                    }
                }
            }
            Message::SelectAll => {
                self.active_menu = ActiveMenu::None;
                // Select the first non-parent entry if available
                if let Some(idx) = self.entries.iter().position(|e| !e.is_parent) {
                    self.selected_entry = Some(idx);
                }
            }
            Message::OpenInTerminal(path_opt) => {
                self.active_menu = ActiveMenu::None;
                let dir = path_opt.unwrap_or_else(|| self.current_path.clone());
                let dir_path = if dir.is_dir() { dir } else { dir.parent().unwrap_or(&self.current_path).to_path_buf() };
                // Try common terminal emulators
                let terminals = ["x-terminal-emulator", "gnome-terminal", "konsole", "xfce4-terminal", "alacritty", "kitty", "xterm"];
                let mut launched = false;
                for term in terminals {
                    if let Ok(_) = std::process::Command::new(term)
                        .current_dir(&dir_path)
                        .arg("--working-directory")
                        .arg(&dir_path)
                        .spawn()
                    {
                        launched = true;
                        break;
                    }
                    // Some terminals don't support --working-directory, try without
                    if let Ok(_) = std::process::Command::new(term)
                        .current_dir(&dir_path)
                        .spawn()
                    {
                        launched = true;
                        break;
                    }
                }
                if !launched {
                    self.error_notification = Some((
                        "No terminal emulator found".to_string(),
                        Instant::now(),
                    ));
                }
            }
            Message::PasteClipboard(target_dir_opt) => {
                let current_dir = target_dir_opt.unwrap_or_else(|| self.current_path.clone());
                if let Some((src_path, is_cut)) = &self.clipboard {
                    if !src_path.exists() {
                        self.error_notification = Some((
                            format!("Source no longer exists: {}", src_path.display()),
                            Instant::now(),
                        ));
                        self.clipboard = None;
                        return Task::none();
                    }
                    if let Some(file_name) = src_path.file_name() {
                        let mut dest_name = file_name.to_str().unwrap_or("file").to_string();
                        let mut counter = 2;
                        while current_dir.join(&dest_name).exists() && !*is_cut {
                            dest_name = format!("{} (copy {})", file_name.to_string_lossy(), counter);
                            counter += 1;
                        }
                        
                        let dest_path = current_dir.join(&dest_name);
                        
                        let (total_size, total_count) = transfer::get_size_and_count(&src_path);
                        if total_size >= 50 * 1024 * 1024 { // >= 50MB — show transfer overlay
                            let info = Arc::new(Mutex::new(transfer::TransferInfo {
                                src: src_path.to_string_lossy().to_string(),
                                dst: dest_path.to_string_lossy().to_string(),
                                bytes_moved: 0,
                                bytes_total: total_size,
                                files_moved: 0,
                                files_total: total_count,
                                history: vec![],
                                current_mbs: 0.0,
                                is_finished: false,
                                is_cut: *is_cut,
                            }));
                            self.active_transfer = Some(info.clone());
                            self.transfer_abort_flag = Some(transfer::start_transfer(src_path.clone(), dest_path, *is_cut, info));
                            
                            if *is_cut {
                                self.clipboard = None;
                            }
                            return Task::none();
                        }
                        
                        // Small file — copy/move inline
                        let result = if *is_cut {
                            match std::fs::rename(src_path, &dest_path) {
                                Ok(_) => { self.clipboard = None; Ok(()) }
                                Err(e) => Err(format!("Move failed: {}", e))
                            }
                        } else {
                            if src_path.is_dir() {
                                match std::process::Command::new("cp")
                                    .arg("-r")
                                    .arg(src_path)
                                    .arg(&dest_path)
                                    .status() {
                                    Ok(status) if status.success() => Ok(()),
                                    Ok(status) => Err(format!("Copy failed with exit code: {}", status)),
                                    Err(e) => Err(format!("Copy failed: {}", e)),
                                }
                            } else {
                                match std::fs::copy(src_path, &dest_path) {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(format!("Copy failed: {}", e)),
                                }
                            }
                        };
                        
                        if let Err(msg) = result {
                            self.error_notification = Some((msg, Instant::now()));
                        }
                        self.refresh_entries();
                    }
                }
            }
            Message::TickTransfer(_) => {
                // Auto-dismiss notifications after 3 seconds
                if let Some((_, time)) = &self.error_notification {
                    if time.elapsed().as_secs() >= 3 {
                        self.error_notification = None;
                    }
                }
                if let Some((_, time)) = &self.clipboard_notification {
                    if time.elapsed().as_secs() >= 2 {
                        self.clipboard_notification = None;
                    }
                }
                if let Some(info_mutex) = &self.active_transfer {
                    let mut is_finished = false;
                    if let Ok(info) = info_mutex.lock() {
                        is_finished = info.is_finished;
                    }
                    if is_finished {
                        self.active_transfer = None;
                        self.transfer_abort_flag = None;
                        self.refresh_entries();
                    }
                }
            }
            Message::AbortTransfer => {
                if let Some(abort) = &self.transfer_abort_flag {
                    abort.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                self.active_transfer = None;
                self.transfer_abort_flag = None;
                self.refresh_entries();
            }
            Message::DismissNotification => {
                self.error_notification = None;
                self.clipboard_notification = None;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let header = self.view_header();
        let main_content = self.view_main();
        let layout = column![header, main_content];

        let base = container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::container_dark);

        let mut stack = iced::widget::Stack::new().push(base);
        
        if self.active_menu != ActiveMenu::None {
            let dropdown = match self.active_menu {
                ActiveMenu::File => self.view_file_dropdown(),
                ActiveMenu::Edit => self.view_edit_dropdown(),
                ActiveMenu::View => self.view_view_dropdown(),
                ActiveMenu::Sort => self.view_sort_dropdown(),
                ActiveMenu::None => unreachable!(),
            };
            stack = stack.push(dropdown);
        }

        if let (Some(idx), Some(pos)) = (self.dragging_item, self.current_drag_pos) {
            if let Some(entry) = self.entries.get(idx) {
                let drag_label = text(entry.display_name())
                    .size(14)
                    .font(mono_font())
                    .color(Color::WHITE);

                let drag_box = container(drag_label)
                    .padding(Padding::from([8, 12]))
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(Color { r: 0.1, g: 0.1, b: 0.1, a: 0.8 })),
                        border: iced::Border {
                            color: theme::BORDER_DIM,
                            width: 1.0,
                            radius: 4.into(),
                        },
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    });

                // Offset the preview slighty so it's under the cursor
                let preview = container(
                    column![
                        Space::new(0, pos.y),
                        row![Space::new(pos.x, 0), drag_box]
                    ]
                )
                .width(Length::Fill)
                .height(Length::Fill);

                stack = stack.push(preview);
            }
        }

        if let Some((idx_opt, pos)) = self.context_menu {
            // Validate index is still within bounds (entries may have changed)
            let idx_opt = idx_opt.filter(|&idx| idx < self.entries.len());
            let context_menu = self.view_context_menu(idx_opt);
            
            // Transparent backdrop to dismiss context menu on click
            let backdrop = button(Space::new(Length::Fill, Length::Fill))
                .style(theme::button_context_backdrop)
                .on_press(Message::GlobalEvent(Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))))
                .width(Length::Fill)
                .height(Length::Fill);
            
            stack = stack.push(backdrop);
            
            let menu_container = container(
                column![
                    Space::new(0, pos.y),
                    row![Space::new(pos.x, 0), context_menu]
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill);

            stack = stack.push(menu_container);
        }

        // Properties overlay
        if let Some(idx) = self.show_properties {
            if let Some(entry) = self.entries.get(idx) {
                let props_overlay = self.view_properties_overlay(idx, entry);
                let props_container = container(props_overlay)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(Background::Color(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.85 })),
                        ..Default::default()
                    });
                stack = stack.push(props_container);
            } else {
                // Index became invalid, clear it
            }
        }

        if let Some(info_mutex) = &self.active_transfer {
            if let Ok(info) = info_mutex.lock() {
                let overlay = self.view_transfer_overlay(&info);
                let overlay_container = container(overlay)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_t| container::Style {
                        background: Some(Background::Color(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.85 })),
                        ..Default::default()
                    });
                stack = stack.push(overlay_container);
            }
        }

        // Error notification toast
        if let Some((ref msg, _)) = self.error_notification {
            let toast = container(
                row![
                    text("⚠ ERROR").size(11).font(mono_font()).color(theme::RED_ACCENT),
                    Space::new(12, 0),
                    text(msg.as_str()).size(11).font(mono_font()).color(Color::WHITE),
                ]
                .align_y(Alignment::Center)
                .padding(Padding::from([10, 16]))
            )
            .style(|_t| container::Style {
                background: Some(Background::Color(Color { r: 0.15, g: 0.02, b: 0.02, a: 0.95 })),
                border: iced::Border { color: theme::RED_ACCENT, width: 1.0, radius: 2.into() },
                shadow: iced::Shadow::default(),
                ..Default::default()
            });

            let toast_layer = container(
                column![
                    Space::new(0, Length::Fill),
                    row![horizontal_space(), toast, Space::new(20, 0)],
                    Space::new(0, 20),
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill);
            stack = stack.push(toast_layer);
        }

        // Clipboard notification toast
        if let Some((ref msg, _)) = self.clipboard_notification {
            let toast = container(
                row![
                    text("✓").size(11).font(mono_font()).color(theme::GREEN_ACCENT),
                    Space::new(8, 0),
                    text(msg.as_str()).size(11).font(mono_font()).color(Color::WHITE),
                ]
                .align_y(Alignment::Center)
                .padding(Padding::from([8, 14]))
            )
            .style(|_t| container::Style {
                background: Some(Background::Color(Color { r: 0.02, g: 0.1, b: 0.02, a: 0.95 })),
                border: iced::Border { color: theme::GREEN_ACCENT, width: 1.0, radius: 2.into() },
                shadow: iced::Shadow::default(),
                ..Default::default()
            });

            let toast_layer = container(
                column![
                    Space::new(0, Length::Fill),
                    row![horizontal_space(), toast, Space::new(20, 0)],
                    Space::new(0, 20),
                ]
            )
            .width(Length::Fill)
            .height(Length::Fill);
            stack = stack.push(toast_layer);
        }

        stack.into()
    }

    fn view_transfer_overlay(&self, info: &transfer::TransferInfo) -> Element<'_, Message> {
        let title_header = row![
            svg(svg::Handle::from_memory(ICON_ORBITAL)).width(14).height(14),
            Space::new(8, 0),
            text("[ ACTION :: DATA_TRANSFER ]").size(16).font(mono_font()).style(|_t| text::Style { color: Some(Color::WHITE) }),
            horizontal_space(),
            text(format!("SYSTEM_TIME: {}", "14:22:09:004")).size(10).color(theme::TEXT_MUTED).font(mono_font()),
        ].align_y(Alignment::Center).padding(Padding::from([20, 20]));
        
        let header_divider = container(Space::new(Length::Fill, 1)).style(|_t| { container::Style { background: Some(Background::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.1 })), ..Default::default() } }).width(Length::Fill);

        let op_info = row![
            column![
                text("CURRENT OPERATION").size(10).color(theme::TEXT_MUTED).font(mono_font()),
                Space::new(0, 4),
                row![
                    container(Space::new(8, 8)).style(|_t| { container::Style { background: Some(Background::Color(theme::TEXT_MUTED)), ..Default::default() } }),
                    Space::new(6, 0),
                    text("TRANSFER_IN_PROGRESS").size(14).color(Color::WHITE).font(mono_font()),
                ].align_y(Alignment::Center)
            ],
            horizontal_space(),
            column![
                row![
                    text("SRC:").size(10).color(theme::TEXT_MUTED).font(mono_font()).width(40),
                    text(info.src.clone()).size(10).color(theme::TEXT_MUTED).font(mono_font())
                ],
                Space::new(0, 4),
                row![
                    text("DST:").size(10).color(theme::TEXT_MUTED).font(mono_font()).width(40),
                    text(info.dst.clone()).size(10).color(theme::TEXT_MUTED).font(mono_font())
                ]
            ]
        ].padding(Padding::from([16, 20]));

        let mbs = info.current_mbs;
        let speed_section = column![
            row![
                column![
                    text("REAL-TIME THROUGHPUT").size(9).color(theme::TEXT_MUTED).font(mono_font()),
                    row![
                        text(format!("{:.0}", mbs)).size(32).color(Color::WHITE).font(mono_font()),
                        Space::new(4, 0),
                        text("MB/s").size(12).color(theme::TEXT_MUTED).font(mono_font()),
                    ].align_y(Alignment::End)
                ],
                horizontal_space(),
                column![
                    text("NETWORK STABILITY").size(9).color(theme::TEXT_MUTED).font(mono_font()),
                    text("99.98% NOMINAL").size(12).color(Color::WHITE).font(mono_font())
                ].align_x(Alignment::End)
            ]
        ].padding(Padding::from([16, 20]));
        
        let mut chart_bars = row![].spacing(2).align_y(Alignment::End).height(Length::Fixed(80.0));
        let max_history = info.history.iter().fold(10.0_f64, |a, &b| a.max(b));
        for i in 0..15 {
            let val = if i < info.history.len() { info.history[i] } else { 0.0 };
            let ratio = (val / max_history) as f32;
            let bar_height = 80.0 * ratio.max(0.1);
            let bar_color = if val == info.current_mbs && val > 0.0 { Color::WHITE } else { Color { r: 1.0, g: 1.0, b: 1.0, a: 0.2 } };
            
            let bar = container(Space::new(Length::Fill, bar_height))
                .style(move |_t| container::Style { background: Some(Background::Color(bar_color)), ..Default::default() })
                .width(Length::FillPortion(1))
                .height(Length::Fixed(bar_height));
            chart_bars = chart_bars.push(bar);
        }
        
        let chart_section = container(
            column![
                speed_section,
                container(chart_bars).padding(Padding::from([0, 20])),
                Space::new(0, 8),
                row![
                    text("T-60s").size(9).color(theme::TEXT_MUTED).font(mono_font()),
                    horizontal_space(),
                    text("T-45s").size(9).color(theme::TEXT_MUTED).font(mono_font()),
                    horizontal_space(),
                    text("T-30s").size(9).color(theme::TEXT_MUTED).font(mono_font()),
                    horizontal_space(),
                    text("T-15s").size(9).color(theme::TEXT_MUTED).font(mono_font()),
                    horizontal_space(),
                    text("LIVE_STREAM").size(9).color(Color::WHITE).font(mono_font()),
                ].padding(Padding::from([0, 20]))
            ]
        ).style(|_t| container::Style {
            border: iced::Border { color: Color { r: 1.0, g: 1.0, b: 1.0, a: 0.1 }, width: 1.0, radius: 2.into() },
            ..Default::default()
        }).padding(Padding { top: 0.0, right: 0.0, bottom: 10.0, left: 0.0 }).width(Length::Fill);

        let pct = if info.bytes_total > 0 { ((info.bytes_moved as f64 / info.bytes_total as f64) * 100.0) as u8 } else { 0 };
        let progress_section = column![
            row![
                text(format!("MOVED: {:.2}GB", info.bytes_moved as f64 / 1_073_741_824.0)).size(10).color(theme::TEXT_MUTED).font(mono_font()),
                Space::new(12, 0),
                text(format!("TOTAL: {:.2}GB", info.bytes_total as f64 / 1_073_741_824.0)).size(10).color(theme::TEXT_MUTED).font(mono_font()),
                horizontal_space(),
                text(format!("{}% COMPLETE", pct)).size(12).color(Color::WHITE).font(mono_font()),
            ],
            Space::new(0, 8),
            container(
                container(Space::new(Length::FillPortion((pct.max(1)) as u16), 8))
                    .style(|_t| container::Style { background: Some(Background::Color(Color { r: 0.8, g: 0.8, b: 0.8, a: 1.0 })), ..Default::default() })
                    .height(Length::Fixed(8.0))
            )
            .style(|_t| container::Style { border: iced::Border { color: Color { r: 1.0, g: 1.0, b: 1.0, a: 0.2 }, width: 1.0, radius: 0.into() }, ..Default::default() })
            .width(Length::Fill).height(Length::Fixed(10.0)).padding(1)
        ].padding(Padding::from([20, 20]));

        let est_time = if info.current_mbs > 0.0 {
            let left_mb = (info.bytes_total - info.bytes_moved) as f64 / 1_048_576.0;
            let secs = (left_mb / info.current_mbs) as u64;
            format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60)
        } else { "00:00:00".to_string() };

        let stats_row = row![
            container(column![text("ESTIMATED TIME").size(9).color(theme::TEXT_MUTED).font(mono_font()), Space::new(0, 6), text(est_time).size(16).color(Color::WHITE).font(mono_font())].align_x(Alignment::Center)).width(Length::FillPortion(1)),
            container(Space::new(1, 40)).style(|_t| container::Style { background: Some(Background::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.1 })), ..Default::default() }),
            container(column![text("FILE QUEUE").size(9).color(theme::TEXT_MUTED).font(mono_font()), Space::new(0, 6), text(format!("{:02} / {:02}", info.files_moved, info.files_total)).size(16).color(Color::WHITE).font(mono_font())].align_x(Alignment::Center)).width(Length::FillPortion(1)),
            container(Space::new(1, 40)).style(|_t| container::Style { background: Some(Background::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.1 })), ..Default::default() }),
            container(column![text("ERROR RATE").size(9).color(theme::TEXT_MUTED).font(mono_font()), Space::new(0, 6), text("0.00%").size(16).color(Color::WHITE).font(mono_font())].align_x(Alignment::Center)).width(Length::FillPortion(1)),
        ].align_y(Alignment::Center).padding(Padding::from([20, 0]));

        let stats_container = container(stats_row).style(|_t| container::Style { border: iced::Border { color: Color { r: 1.0, g: 1.0, b: 1.0, a: 0.1 }, width: 1.0, radius: 0.into() }, ..Default::default() }).width(Length::Fill);

        let abort_btn = button(text("[ ABORT_OPERATION ]").size(12).font(mono_font()).color(Color::WHITE))
            .style(|_t, _s| button::Style { border: iced::Border { color: Color { r: 1.0, g: 1.0, b: 1.0, a: 0.3 }, width: 1.0, radius: 2.into() }, background: Some(Background::Color(Color::TRANSPARENT)), text_color: Color::WHITE, shadow: iced::Shadow::default() })
            .padding(Padding::from([8, 16]))
            .on_press(Message::AbortTransfer);

        let footer = row![
            row![
                container(Space::new(8, 8)).style(|_t| container::Style { background: Some(Background::Color(theme::GREEN_ACCENT)), ..Default::default() }),
                Space::new(4, 0),
                text("SECURE_LINK").size(9).color(theme::TEXT_MUTED).font(mono_font()),
            ].align_y(Alignment::Center),
            Space::new(12, 0),
            row![
                container(Space::new(8, 8)).style(|_t| container::Style { background: Some(Background::Color(Color::WHITE)), ..Default::default() }),
                Space::new(4, 0),
                text("ENCRYPT_AES256").size(9).color(theme::TEXT_MUTED).font(mono_font()),
            ].align_y(Alignment::Center),
            horizontal_space(),
            abort_btn
        ].align_y(Alignment::Center).padding(Padding::from([20, 20]));

        let inner_window = column![title_header, header_divider, op_info, container(chart_section).padding(Padding::from([0, 20])), progress_section, container(stats_container).padding(Padding::from([0, 20])), footer];
        
        container(inner_window)
            .width(800)
            .style(|_t| container::Style { background: Some(Background::Color(Color { r: 0.02, g: 0.02, b: 0.02, a: 1.0 })), border: iced::Border { color: Color { r: 1.0, g: 1.0, b: 1.0, a: 0.2 }, width: 1.0, radius: 0.into() }, shadow: iced::Shadow::default(), ..Default::default() })
            .into()
    }

    fn view_context_menu(&self, target_idx: Option<usize>) -> Element<'_, Message> {
        // Clamp invalid index to None to prevent out-of-bounds panics
        let target_idx = target_idx.filter(|&idx| idx < self.entries.len());
        let is_dir = target_idx.map_or(false, |idx| self.entries[idx].is_dir);
        let is_parent = target_idx.map_or(false, |idx| self.entries[idx].is_parent);
        let has_target = target_idx.is_some();
        
        // Header — shows what was clicked
        let header_label = if !has_target {
            "DIRECTORY_CONTEXT"
        } else if is_parent {
            "PARENT_DIR_CONTEXT"
        } else if is_dir {
            "FOLDER_CONTEXT"
        } else {
            "FILE_CONTEXT"
        };
        
        let header = row![
            text(header_label).size(11).font(mono_font()).color(theme::TEXT_MUTED),
            horizontal_space(),
            text("UID: 1000").size(11).font(mono_font()).color(theme::TEXT_MUTED),
        ]
        .padding(Padding { top: 8.0, right: 12.0, bottom: 8.0, left: 12.0 });

        let divider = || container(
            container(Space::new(Length::Fill, 1))
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.08 })),
                    ..Default::default()
                })
                .width(Length::Fill)
        )
        .width(Length::Fill)
        .padding(Padding { top: 4.0, right: 12.0, bottom: 4.0, left: 12.0 });

        // Actions column
        let mut actions = column![header, divider()].spacing(0);

        let action_btn = |icon: &'static [u8], label: &'static str, shortcut: &'static str, msg: Message, is_danger: bool| {
            let icon_svg = svg(svg::Handle::from_memory(icon))
                .width(Length::Fixed(16.0))
                .height(Length::Fixed(16.0));
            
            let mut label_text = text(label).size(13).font(mono_font());
            let mut shortcut_text = text(shortcut).size(11).font(mono_font());
            
            if is_danger {
                label_text = label_text.color(theme::RED_ACCENT);
                shortcut_text = shortcut_text.color(theme::RED_ACCENT);
            } else {
                label_text = label_text.color(Color::WHITE);
                shortcut_text = shortcut_text.color(theme::TEXT_MUTED);
            }

            let content = row![
                icon_svg,
                Space::new(12, 0),
                label_text,
                horizontal_space(),
                shortcut_text,
            ]
            .align_y(Alignment::Center)
            .padding(Padding { top: 8.0, right: 16.0, bottom: 8.0, left: 16.0 });

            button(content)
                .width(Length::Fill)
                .style(if is_danger { theme::button_context_danger as fn(&Theme, button::Status) -> button::Style } else { theme::button_context_menu as fn(&Theme, button::Status) -> button::Style })
                .on_press(msg)
        };

        if has_target && !is_parent {
            // ── File or Folder target ──
            if is_dir {
                actions = actions.push(action_btn(ICON_OPEN, "OPEN", "ENTER", Message::ContextMenuAction("OPEN".to_string(), target_idx), false));
                actions = actions.push(action_btn(ICON_TERMINAL, "OPEN IN TERMINAL", "", Message::ContextMenuAction("OPEN_TERMINAL".to_string(), target_idx), false));
            } else {
                actions = actions.push(action_btn(ICON_OPEN, "OPEN", "ENTER", Message::ContextMenuAction("OPEN".to_string(), target_idx), false));
                actions = actions.push(action_btn(ICON_OPEN_WITH, "OPEN WITH...", "", Message::ContextMenuAction("OPEN".to_string(), target_idx), false));
            }
            
            actions = actions.push(divider());
            actions = actions.push(action_btn(ICON_RENAME, "RENAME", "F2", Message::ContextMenuAction("RENAME".to_string(), target_idx), false));
            actions = actions.push(divider());
            actions = actions.push(action_btn(ICON_COPY, "COPY", "CTRL+C", Message::ContextMenuAction("COPY".to_string(), target_idx), false));
            actions = actions.push(action_btn(ICON_CUT, "CUT", "CTRL+X", Message::ContextMenuAction("CUT".to_string(), target_idx), false));
            
            if self.clipboard.is_some() {
                actions = actions.push(action_btn(ICON_PASTE, "PASTE", "CTRL+V", Message::ContextMenuAction("PASTE".to_string(), target_idx), false));
            }
            
            actions = actions.push(divider());
            actions = actions.push(action_btn(ICON_DELETE, "DELETE", "DEL", Message::ContextMenuAction("DELETE".to_string(), target_idx), true));
            actions = actions.push(divider());
            actions = actions.push(action_btn(ICON_PROPERTIES, "PROPERTIES", "ALT+ENTER", Message::ContextMenuAction("PROPERTIES".to_string(), target_idx), false));
        } else if is_parent {
            // ── Parent directory (..) target ──
            actions = actions.push(action_btn(ICON_OPEN, "OPEN", "ENTER", Message::ContextMenuAction("OPEN".to_string(), target_idx), false));
            actions = actions.push(action_btn(ICON_TERMINAL, "OPEN IN TERMINAL", "", Message::ContextMenuAction("OPEN_TERMINAL".to_string(), target_idx), false));
            if self.clipboard.is_some() {
                actions = actions.push(divider());
                actions = actions.push(action_btn(ICON_PASTE, "PASTE HERE", "CTRL+V", Message::ContextMenuAction("PASTE".to_string(), target_idx), false));
            }
        } else {
            // ── Empty space (no target) ──
            actions = actions.push(action_btn(ICON_NEW_FOLDER, "NEW FOLDER", "SHIFT+N", Message::ContextMenuAction("NEW_FOLDER".to_string(), None), false));
            actions = actions.push(divider());
            if self.clipboard.is_some() {
                actions = actions.push(action_btn(ICON_PASTE, "PASTE", "CTRL+V", Message::ContextMenuAction("PASTE".to_string(), None), false));
                actions = actions.push(divider());
            }
            actions = actions.push(action_btn(ICON_SELECT_ALL, "SELECT ALL", "CTRL+A", Message::ContextMenuAction("SELECT_ALL".to_string(), None), false));
            actions = actions.push(action_btn(ICON_TERMINAL, "OPEN IN TERMINAL", "", Message::ContextMenuAction("OPEN_TERMINAL".to_string(), None), false));
            actions = actions.push(divider());
            actions = actions.push(action_btn(ICON_REFRESH, "REFRESH", "F5", Message::ContextMenuAction("REFRESH".to_string(), None), false));
        }

        // Footer
        let obj_id = if !has_target { "SYSTEM_DIR_CURRENT" } 
            else if is_parent { "SYSTEM_DIR_PARENT" } 
            else if is_dir { "SYSTEM_DIR_01" } 
            else { "SYSTEM_DATA_01" };
        
        let footer = row![
            text(format!("OBJ: {}", obj_id)).size(10).font(mono_font()).color(theme::TEXT_MUTED),
            horizontal_space(),
            text("SEC: HIGH").size(10).font(mono_font()).color(theme::TEXT_MUTED),
        ]
        .padding(Padding { top: 6.0, right: 12.0, bottom: 6.0, left: 12.0 });
        
        actions = actions.push(divider());
        actions = actions.push(footer);

        container(actions)
            .width(Length::Fixed(300.0))
            .style(|_theme| container::Style {
                background: Some(Background::Color(Color { r: 0.04, g: 0.04, b: 0.04, a: 0.98 })),
                border: iced::Border {
                    color: Color { r: 1.0, g: 1.0, b: 1.0, a: 0.15 },
                    width: 1.0,
                    radius: 4.into(),
                },
                shadow: iced::Shadow::default(),
                ..Default::default()
            })
            .into()
    }

    fn view_properties_overlay(&self, _idx: usize, entry: &FileEntry) -> Element<'_, Message> {
        let name = if entry.is_dir {
            entry.name.trim_end_matches('/').to_string()
        } else {
            entry.name.clone()
        };
        let path = self.current_path.join(&name);
        
        let type_label = if entry.is_parent { "Parent Directory" }
            else if entry.is_dir { "Directory" }
            else if entry.is_executable { "Executable File" }
            else { "Regular File" };

        let title_header = row![
            svg(svg::Handle::from_memory(ICON_PROPERTIES)).width(14).height(14),
            Space::new(8, 0),
            text("[ PROPERTIES ]").size(16).font(mono_font()).style(|_t| text::Style { color: Some(Color::WHITE) }),
            horizontal_space(),
            button(text("[ CLOSE ]").size(11).font(mono_font()).color(theme::TEXT_MUTED))
                .style(theme::button_context_menu)
                .padding(Padding::from([4, 8]))
                .on_press(Message::CloseMenus),
        ].align_y(Alignment::Center).padding(Padding::from([16, 20]));

        let header_divider = container(Space::new(Length::Fill, 1)).style(|_t| { container::Style { background: Some(Background::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.1 })), ..Default::default() } }).width(Length::Fill);
        
        let prop_row = |label: &str, value: String| -> Element<'_, Message> {
            let label = label.to_string();
            row![
                text(label).size(11).font(mono_font()).color(theme::TEXT_MUTED).width(120),
                text(value).size(11).font(mono_font()).color(Color::WHITE),
            ]
            .align_y(Alignment::Center)
            .padding(Padding::from([6, 20]))
            .into()
        };
        
        let icon_bytes = if entry.is_parent { ICON_PARENT }
            else if entry.is_dir { ICON_FOLDER }
            else if entry.is_executable { ICON_EXEC }
            else { ICON_FILE };

        let icon_row = row![
            svg(svg::Handle::from_memory(icon_bytes)).width(32).height(32),
            Space::new(12, 0),
            column![
                text(name.clone()).size(14).font(mono_font()).color(Color::WHITE),
                text(type_label).size(11).font(mono_font()).color(theme::TEXT_MUTED),
            ],
        ].align_y(Alignment::Center).padding(Padding::from([16, 20]));

        let props_content = column![
            title_header,
            header_divider,
            icon_row,
            container(Space::new(Length::Fill, 1)).style(|_t| { container::Style { background: Some(Background::Color(Color { r: 1.0, g: 1.0, b: 1.0, a: 0.06 })), ..Default::default() } }).width(Length::Fill),
            prop_row("NAME:", name),
            prop_row("TYPE:", type_label.to_string()),
            prop_row("PATH:", path.display().to_string()),
            prop_row("SIZE:", entry.format_size()),
            prop_row("PERMISSIONS:", entry.permissions.clone()),
            prop_row("MODIFIED:", entry.format_date()),
        ];

        let close_btn = button(text("[ OK ]").size(12).font(mono_font()).color(Color::WHITE))
            .style(|_t, _s| button::Style { 
                border: iced::Border { color: Color { r: 1.0, g: 1.0, b: 1.0, a: 0.3 }, width: 1.0, radius: 2.into() }, 
                background: Some(Background::Color(Color::TRANSPARENT)), 
                text_color: Color::WHITE, 
                shadow: iced::Shadow::default() 
            })
            .padding(Padding::from([6, 20]))
            .on_press(Message::CloseMenus);

        let footer = container(row![horizontal_space(), close_btn].padding(Padding::from([12, 20]))).width(Length::Fill);

        container(column![props_content, Space::new(0, Length::Fill), footer])
            .width(480)
            .height(400)
            .style(|_t| container::Style { 
                background: Some(Background::Color(Color { r: 0.02, g: 0.02, b: 0.02, a: 1.0 })), 
                border: iced::Border { color: Color { r: 1.0, g: 1.0, b: 1.0, a: 0.2 }, width: 1.0, radius: 2.into() }, 
                shadow: iced::Shadow::default(), 
                ..Default::default() 
            })
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    // ─── Header ──────────────────────────────────────────────────────────────
    fn view_header(&self) -> Element<'_, Message> {
        let logo_handle = svg::Handle::from_memory(ICON_ORBITAL);
        let logo_svg = svg(logo_handle).width(30).height(30);

        let logo = row![
            logo_svg,
            text("ORBITAL")
                .size(16)
                .color(theme::TEXT_PRIMARY)
                .font(mono_font())
                .style(|_theme| text::Style {
                    color: Some(theme::TEXT_PRIMARY),
                }),
        ]
        .spacing(12)
        .align_y(Alignment::Center);

        let sep = container(Space::new(1, 24))
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(theme::BORDER_DIM)),
                ..Default::default()
            })
            .width(1);

        let nav_btn = |label: &str, menu: ActiveMenu| -> Element<'_, Message> {
            let label = label.to_string();
            let style = if self.active_menu == menu {
                theme::button_nav_active as fn(&Theme, button::Status) -> button::Style
            } else {
                theme::button_nav
            };
            button(text(label).size(12).font(mono_font()))
                .style(style)
                .padding(Padding::from([4, 8]))
                .on_press(Message::ToggleMenu(menu))
                .into()
        };

        let nav = row![
            nav_btn("FILE", ActiveMenu::File),
            nav_btn("EDIT", ActiveMenu::Edit),
            nav_btn("VIEW", ActiveMenu::View),
            nav_btn("SORT", ActiveMenu::Sort),
            button(text("GO").size(12).font(mono_font()))
                .style(theme::button_nav)
                .padding(Padding::from([4, 8]))
                .on_press(Message::CloseMenus),
        ]
        .spacing(16)
        .align_y(Alignment::Center);

        let left = row![logo, sep, nav]
            .spacing(16)
            .align_y(Alignment::Center);

        let search = text_input("SEARCH SYSTEM...", &self.search_query)
            .on_input(Message::SearchChanged)
            .size(12)
            .font(mono_font())
            .width(220)
            .padding(Padding::from([6, 10]))
            .style(theme::text_input_dark);

        let status_dot = container(Space::new(8, 8))
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(theme::GREEN_ACCENT)),
                border: iced::Border { radius: 4.into(), ..Default::default() },
                ..Default::default()
            });

        let status = row![
            status_dot,
            text("ONLINE").size(10).color(theme::TEXT_MUTED).font(mono_font()),
        ]
        .spacing(6)
        .align_y(Alignment::Center);

        let settings_btn = button(text("[=]").size(14).font(mono_font()))
            .style(theme::button_settings)
            .padding(Padding::from([6, 8]))
            .on_press(Message::CloseMenus);

        let right = row![search, status, settings_btn]
            .spacing(12)
            .align_y(Alignment::Center);

        let header_row = row![left, horizontal_space(), right]
            .align_y(Alignment::Center)
            .padding(Padding::from([12, 20]));

        container(
            column![
                header_row,
                container(Space::new(Length::Fill, 1))
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(theme::BORDER_DIM)),
                        ..Default::default()
                    })
                    .width(Length::Fill),
            ]
        )
        .style(theme::container_header)
        .width(Length::Fill)
        .into()
    }

    // ─── Dropdown positioning helper ─────────────────────────────────────────
    fn wrap_dropdown<'a>(&self, menu: ActiveMenu, panel: Element<'a, Message>) -> Element<'a, Message> {
        // Compute horizontal offset based on which menu is active
        let left_offset: u16 = match menu {
            ActiveMenu::File => 150,
            ActiveMenu::Edit => 210,
            ActiveMenu::View => 270,
            ActiveMenu::Sort => 330,
            ActiveMenu::None => 0,
        };

        // Transparent backdrop that closes menu when clicked outside
        let backdrop = button(Space::new(Length::Fill, Length::Fill))
            .style(theme::button_menu_backdrop)
            .on_press(Message::CloseMenus)
            .width(Length::Fill)
            .height(Length::Fill);

        // Positioned dropdown panel
        let menu_layer = container(
            column![
                Space::new(0, 50),
                row![
                    Space::new(left_offset, 0),
                    panel,
                ],
                Space::new(0, Length::Fill),
            ]
        )
        .width(Length::Fill)
        .height(Length::Fill);

        // Stack: backdrop catches outside clicks, menu panel sits on top
        iced::widget::Stack::new()
            .push(backdrop)
            .push(menu_layer)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // ─── Menu item helpers ───────────────────────────────────────────────────
    fn menu_item(&self, label: &str, shortcut: &str, msg: Message) -> Element<'_, Message> {
        let label = label.to_string();
        let shortcut = shortcut.to_string();
        let content = row![
            text(label).size(13).font(mono_font()).color(theme::TEXT_PRIMARY),
            horizontal_space(),
            text(shortcut).size(11).font(mono_font()).color(theme::TEXT_MUTED),
        ]
        .align_y(Alignment::Center)
        .padding(Padding::from([10, 20]));

        button(content)
            .style(theme::button_menu_item)
            .width(Length::Fill)
            .padding(0)
            .on_press(msg)
            .into()
    }

    fn menu_item_checked(&self, label: &str, shortcut: &str, active: bool, msg: Message) -> Element<'_, Message> {
        let indicator = if active { ">" } else { " " };
        let label = label.to_string();
        let shortcut = shortcut.to_string();

        let label_color = if active { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
        let style = if active {
            theme::button_menu_item_active as fn(&Theme, button::Status) -> button::Style
        } else {
            theme::button_menu_item
        };

        let content = row![
            text(indicator).size(13).font(mono_font()).color(theme::TEXT_PRIMARY),
            Space::new(6, 0),
            text(label).size(13).font(mono_font()).color(label_color),
            horizontal_space(),
            text(shortcut).size(11).font(mono_font()).color(theme::TEXT_MUTED),
        ]
        .align_y(Alignment::Center)
        .padding(Padding::from([10, 20]));

        button(content)
            .style(style)
            .width(Length::Fill)
            .padding(0)
            .on_press(msg)
            .into()
    }

    fn menu_item_danger(&self, label: &str, shortcut: &str, msg: Message) -> Element<'_, Message> {
        let label = label.to_string();
        let shortcut = shortcut.to_string();
        let content = row![
            text(label).size(13).font(mono_font()).color(theme::RED_ACCENT),
            horizontal_space(),
            text(shortcut).size(11).font(mono_font()).color(theme::TEXT_MUTED),
        ]
        .align_y(Alignment::Center)
        .padding(Padding::from([10, 20]));

        button(content)
            .style(theme::button_menu_item_danger)
            .width(Length::Fill)
            .padding(0)
            .on_press(msg)
            .into()
    }

    fn menu_divider(&self) -> Element<'_, Message> {
        container(
            container(Space::new(Length::Fill, 1))
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(theme::BORDER_DIM)),
                    ..Default::default()
                })
                .width(Length::Fill),
        )
        .padding(Padding::from([4, 16]))
        .width(Length::Fill)
        .into()
    }

    fn menu_panel<'a>(&self, items: Element<'a, Message>, footer_left: &str, footer_right: &str) -> Element<'a, Message> {
        let fl = footer_left.to_string();
        let fr = footer_right.to_string();
        let footer_row = row![
            text(fl).size(9).font(mono_font()).color(theme::TEXT_MUTED),
            horizontal_space(),
            text(fr).size(9).font(mono_font()).color(theme::TEXT_MUTED),
        ]
        .padding(Padding::from([8, 16]));

        container(
            column![
                container(items).padding(Padding::from([6, 0])),
                container(Space::new(Length::Fill, 1))
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(theme::BORDER_DIM)),
                        ..Default::default()
                    })
                    .width(Length::Fill),
                footer_row,
            ]
        )
        .style(theme::container_dropdown)
        .width(300)
        .into()
    }

    // ─── FILE dropdown ───────────────────────────────────────────────────────
    fn view_file_dropdown(&self) -> Element<'_, Message> {
        let items: Element<'_, Message> = column![
            self.menu_item("New Tab", "Ctrl + T", Message::Noop),
            self.menu_item("New Window", "Ctrl + N", Message::Noop),
            self.menu_item("New Folder", "Shift + N", Message::NewFolder),
            self.menu_divider(),
            self.menu_item("Rename ...", "F2", Message::StartRename),
            self.menu_divider(),
            self.menu_item("Duplicate", "Ctrl + D", Message::DuplicateEntry),
            self.menu_item("Export Data", "Ctrl + E", Message::Noop),
            self.menu_divider(),
            self.menu_item_danger("Move to Trash", "Del", Message::MoveToTrash),
        ]
        .spacing(0)
        .into();

        let panel = self.menu_panel(items, "ROOT_FS_ALLOCATED", "SECURED");
        self.wrap_dropdown(ActiveMenu::File, panel)
    }

    // ─── EDIT dropdown ───────────────────────────────────────────────────────
    fn view_edit_dropdown(&self) -> Element<'_, Message> {
        let items: Element<'_, Message> = column![
            self.menu_item("Undo", "Ctrl + Z", Message::Noop),
            self.menu_item("Redo", "Ctrl + Y", Message::Noop),
            self.menu_divider(),
            self.menu_item("Cut", "Ctrl + X", Message::CutSelected),
            self.menu_item("Copy", "Ctrl + C", Message::CopySelected),
            self.menu_item("Paste", "Ctrl + V", Message::PasteClipboard(None)),
            self.menu_divider(),
            self.menu_item("Select All", "Ctrl + A", Message::SelectAll),
            self.menu_item("Rename", "F2", Message::StartRename),
        ]
        .spacing(0)
        .into();

        let panel = self.menu_panel(items, "EDIT_BUFFER", "READY");
        self.wrap_dropdown(ActiveMenu::Edit, panel)
    }

    // ─── VIEW dropdown ───────────────────────────────────────────────────────
    fn view_view_dropdown(&self) -> Element<'_, Message> {
        let items: Element<'_, Message> = column![
            self.menu_item_checked("Show Hidden Files", "Ctrl + H", self.show_hidden, Message::ToggleHidden),
            self.menu_divider(),
            self.menu_item("Refresh", "F5", Message::RefreshDir),
            self.menu_item_checked("Toggle Sidebar", "", self.show_sidebar, Message::ToggleSidebar),
        ]
        .spacing(0)
        .into();

        let panel = self.menu_panel(items, "VIEW_ENGINE", "ACTIVE");
        self.wrap_dropdown(ActiveMenu::View, panel)
    }

    // ─── SORT dropdown ───────────────────────────────────────────────────────
    fn view_sort_dropdown(&self) -> Element<'_, Message> {
        let items: Element<'_, Message> = column![
            self.menu_item_checked("Sort by Name", "", self.sort_column == SortColumn::Name, Message::SortBy(SortColumn::Name)),
            self.menu_item_checked("Date Modified", "", self.sort_column == SortColumn::Date, Message::SortBy(SortColumn::Date)),
            self.menu_item_checked("File Type", "", false, Message::Noop),
            self.menu_item_checked("Size", "", self.sort_column == SortColumn::Size, Message::SortBy(SortColumn::Size)),
            self.menu_divider(),
            self.menu_item_checked("Ascending", "", self.sort_ascending, Message::SetSortAscending),
            self.menu_item_checked("Descending", "", !self.sort_ascending, Message::SetSortDescending),
        ]
        .spacing(0)
        .into();

        let panel = self.menu_panel(items, "SORT_INDEX", "ORDERED");
        self.wrap_dropdown(ActiveMenu::Sort, panel)
    }

    // ─── Main content area ───────────────────────────────────────────────────
    fn view_main(&self) -> Element<'_, Message> {
        let content = self.view_content();

        if self.show_sidebar {
            let sidebar = self.view_sidebar();
            let sep = container(Space::new(1, Length::Fill))
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(theme::BORDER_DIM)),
                    ..Default::default()
                })
                .width(1);

            row![sidebar, sep, content]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            content
        }
    }

    // ─── Sidebar ─────────────────────────────────────────────────────────────
    fn view_sidebar(&self) -> Element<'_, Message> {
        let drives_label = text("DRIVES")
            .size(10)
            .color(theme::TEXT_MUTED)
            .font(mono_font());

        let local_btn = self.sidebar_item("Local (C:)", ICON_DRIVE_LOCAL, true, Message::NavigateTo(PathBuf::from("/")));

        let drives = column![
            container(drives_label).padding(Padding::from([0, 14])),
            local_btn,
        ]
        .spacing(2);

        let fav_label = text("FAVORITES")
            .size(10)
            .color(theme::TEXT_MUTED)
            .font(mono_font());

        let home_path = dirs_or_root();
        let projects_btn = self.sidebar_item("Projects", ICON_PROJECTS, false,
            Message::NavigateTo(home_path.join("Projects")));
        let downloads_btn = self.sidebar_item("Downloads", ICON_DOWNLOADS, false,
            Message::NavigateTo(home_path.join("Downloads")));
        let pictures_btn = self.sidebar_item("Pictures", ICON_PICTURES, false,
            Message::NavigateTo(home_path.join("Pictures")));
        let documents_btn = self.sidebar_item("Documents", ICON_DOCUMENTS, false,
            Message::NavigateTo(home_path.join("Documents")));
        let videos_btn = self.sidebar_item("Videos", ICON_VIDEOS, false,
            Message::NavigateTo(home_path.join("Videos")));

        let favorites = column![
            container(fav_label).padding(Padding::from([0, 14])),
            projects_btn,
            downloads_btn,
            pictures_btn,
            documents_btn,
            videos_btn,
        ]
        .spacing(2);

        let stats = self.view_system_stats();

        let sidebar_content = column![
            Space::new(0, 12),
            drives,
            Space::new(0, 16),
            favorites,
            Space::new(0, Length::Fill),
            stats,
        ]
        .width(200);

        container(sidebar_content)
            .height(Length::Fill)
            .style(theme::container_sidebar)
            .into()
    }

    fn sidebar_item(
        &self,
        label: &str,
        icon_bytes: &'static [u8],
        active: bool,
        msg: Message,
    ) -> Element<'_, Message> {
        let label = label.to_string();
        let style = if active {
            theme::button_sidebar_active as fn(&Theme, button::Status) -> button::Style
        } else {
            theme::button_sidebar
        };

        let left_border_color = if active {
            theme::TEXT_PRIMARY
        } else {
            Color::TRANSPARENT
        };

        let border_bar = container(Space::new(2, Length::Fill))
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(left_border_color)),
                ..Default::default()
            })
            .height(Length::Fill);

        let icon_handle = svg::Handle::from_memory(icon_bytes);
        let icon_widget = svg(icon_handle).width(14).height(14);

        let btn_content = row![
            icon_widget,
            text(label).size(13).font(mono_font()),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let btn = button(btn_content)
            .style(style)
            .padding(Padding::from([8, 12]))
            .width(Length::Fill)
            .on_press(msg);

        row![border_bar, btn]
            .height(36)
            .width(Length::Fill)
            .into()
    }

    fn view_system_stats(&self) -> Element<'_, Message> {
        let sep = container(Space::new(Length::Fill, 1))
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(theme::BORDER_DIM)),
                ..Default::default()
            })
            .width(Length::Fill);

        let cpu_label = row![
            text("CPU").size(10).color(theme::TEXT_MUTED).font(mono_font()),
            horizontal_space(),
            text(format!("{}%", self.cpu_usage as u32))
                .size(10).color(theme::TEXT_MUTED).font(mono_font()),
        ]
        .padding(Padding::from([0, 18]));

        let cpu_bar = container(self.stat_bar(self.cpu_usage / 100.0, Color {
            r: 0.35, g: 0.55, b: 0.85, a: 1.0,
        })).padding(Padding::from([0, 18]));

        let mem_label = row![
            text("MEM").size(10).color(theme::TEXT_MUTED).font(mono_font()),
            horizontal_space(),
            text(format!("{}%", self.mem_usage as u32))
                .size(10).color(theme::TEXT_MUTED).font(mono_font()),
        ]
        .padding(Padding::from([0, 18]));

        let mem_bar = container(self.stat_bar(self.mem_usage / 100.0, theme::TEXT_PRIMARY))
            .padding(Padding::from([0, 18]));

        column![
            sep, Space::new(0, 20),
            cpu_label, cpu_bar,
            Space::new(0, 26),
            mem_label, mem_bar,
            Space::new(0, 20),
        ]
        .width(Length::Fill)
        .into()
    }

    fn stat_bar(&self, fraction: f32, fill_color: Color) -> Element<'_, Message> {
        let fill_width = (fraction * 200.0).max(0.0);
        let fill = container(Space::new(fill_width as u16, 3))
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(fill_color)),
                ..Default::default()
            });
        container(fill)
            .width(Length::Fill)
            .height(3)
            .style(theme::container_progress_track)
            .into()
    }

    // ─── Content area ────────────────────────────────────────────────────────
    fn view_content(&self) -> Element<'_, Message> {
        let breadcrumbs = self.view_breadcrumbs();
        let table_header = self.view_table_header();
        let file_list = self.view_file_list();

        let bc_sep = container(Space::new(Length::Fill, 1))
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(theme::BORDER_DIM)),
                ..Default::default()
            })
            .width(Length::Fill);

        let th_sep = container(Space::new(Length::Fill, 1))
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(theme::BORDER_DIM)),
                ..Default::default()
            })
            .width(Length::Fill);

        column![breadcrumbs, bc_sep, table_header, th_sep, file_list]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_breadcrumbs(&self) -> Element<'_, Message> {
        let components: Vec<_> = self.current_path.components().collect();
        let mut crumbs = Row::new().spacing(4).align_y(Alignment::Center);

        for (i, comp) in components.iter().enumerate() {
            let label = match comp {
                std::path::Component::RootDir => "ROOT".to_string(),
                std::path::Component::Normal(name) => name.to_string_lossy().to_uppercase(),
                _ => comp.as_os_str().to_string_lossy().to_uppercase(),
            };
            let is_last = i == components.len() - 1;
            if i > 0 {
                crumbs = crumbs.push(
                    text("/").size(12).color(theme::TEXT_MUTED).font(mono_font()),
                );
            }
            let style = if is_last {
                theme::button_breadcrumb_active as fn(&Theme, button::Status) -> button::Style
            } else {
                theme::button_breadcrumb
            };
            crumbs = crumbs.push(
                button(text(label).size(11).font(mono_font()))
                    .style(style)
                    .padding(Padding::from([4, 10]))
                    .on_press(Message::BreadcrumbClicked(i)),
            );
        }

        container(crumbs)
            .padding(Padding::from([12, 20]))
            .width(Length::Fill)
            .style(theme::container_breadcrumb)
            .into()
    }

    fn view_table_header(&self) -> Element<'_, Message> {
        let hash_col = container(
            text("#").size(10).color(theme::TEXT_MUTED).font(mono_font()),
        )
        .width(50)
        .center_x(50);

        let name_style = if self.sort_column == SortColumn::Name {
            theme::button_column_header_active as fn(&Theme, button::Status) -> button::Style
        } else {
            theme::button_column_header
        };
        let name_label = if self.sort_column == SortColumn::Name {
            if self.sort_ascending { "NAME v" } else { "NAME ^" }
        } else { "NAME" };

        let size_style = if self.sort_column == SortColumn::Size {
            theme::button_column_header_active as fn(&Theme, button::Status) -> button::Style
        } else {
            theme::button_column_header
        };
        let size_label = if self.sort_column == SortColumn::Size {
            if self.sort_ascending { "SIZE v" } else { "SIZE ^" }
        } else { "SIZE" };

        let date_style = if self.sort_column == SortColumn::Date {
            theme::button_column_header_active as fn(&Theme, button::Status) -> button::Style
        } else {
            theme::button_column_header
        };
        let date_label = if self.sort_column == SortColumn::Date {
            if self.sort_ascending { "DATE v" } else { "DATE ^" }
        } else { "DATE" };

        let header_row = row![
            hash_col,
            container(
                button(text(name_label).size(10).font(mono_font()))
                    .style(name_style)
                    .on_press(Message::SortBy(SortColumn::Name))
                    .padding(0)
            ).width(Length::FillPortion(5)),
            container(text("PERMS").size(10).color(theme::TEXT_MUTED).font(mono_font()))
                .width(Length::FillPortion(2)),
            container(
                button(text(size_label).size(10).font(mono_font()))
                    .style(size_style)
                    .on_press(Message::SortBy(SortColumn::Size))
                    .padding(0)
            ).width(Length::FillPortion(2)).align_x(alignment::Horizontal::Right),
            container(
                button(text(date_label).size(10).font(mono_font()))
                    .style(date_style)
                    .on_press(Message::SortBy(SortColumn::Date))
                    .padding(0)
            ).width(Length::FillPortion(2)).align_x(alignment::Horizontal::Right),
        ]
        .spacing(8)
        .padding(Padding::from([8, 20]))
        .align_y(Alignment::Center);

        container(header_row)
            .width(Length::Fill)
            .style(theme::container_dark)
            .into()
    }

    // ─── File list ───────────────────────────────────────────────────────────
    fn view_file_list(&self) -> Element<'_, Message> {
        let search_lower = self.search_query.to_lowercase();
        let filtered: Vec<(usize, &FileEntry)> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                if search_lower.is_empty() { true }
                else { e.name.to_lowercase().contains(&search_lower) }
            })
            .collect();

        let mut list = Column::new().spacing(0);
        for (idx, entry) in filtered {
            list = list.push(self.view_file_row(idx, entry));
        }

        scrollable(
            container(list)
                .width(Length::Fill)
                .style(theme::container_dark),
        )
        .style(theme::scrollable_dark)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_file_row(&self, idx: usize, entry: &FileEntry) -> Element<'_, Message> {
        let is_parent = entry.is_parent;
        let is_dir = entry.is_dir;
        let is_executable = entry.is_executable;
        let display_name = entry.display_name();
        let permissions = entry.permissions.clone();
        let size_str = entry.format_size();
        let date_str = entry.format_date();

        let icon_bytes = if is_parent { ICON_PARENT }
            else if is_dir { ICON_FOLDER }
            else if is_executable { ICON_EXEC }
            else { ICON_FILE };

        let icon_handle = svg::Handle::from_memory(icon_bytes);
        let icon = container(svg(icon_handle).width(16).height(16))
            .width(50).center_x(50);

        // Inline rename
        let is_renaming = self.renaming && self.selected_entry == Some(idx);
        let name_widget: Element<'_, Message> = if is_renaming {
            text_input("", &self.rename_input)
                .id(text_input::Id::new("rename_input"))
                .on_input(Message::RenameInputChanged)
                .on_submit(Message::ConfirmRename)
                .size(14)
                .font(mono_font())
                .padding(Padding::from([2, 6]))
                .style(theme::text_input_rename)
                .into()
        } else {
            text(display_name).size(14).font(mono_font()).into()
        };

        let half_white = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.5 };

        let row_content = row![
            icon,
            container(name_widget).width(Length::FillPortion(5)),
            container(text(permissions).size(12).color(half_white).font(mono_font()))
                .width(Length::FillPortion(2)),
            container(text(size_str).size(12).color(half_white).font(mono_font()))
                .width(Length::FillPortion(2))
                .align_x(alignment::Horizontal::Right),
            container(text(date_str).size(12).color(half_white).font(mono_font()))
                .width(Length::FillPortion(2))
                .align_x(alignment::Horizontal::Right),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let row_with_border = column![
            container(row_content)
                .padding(Padding::from([10, 20]))
                .width(Length::Fill),
            container(Space::new(Length::Fill, 1))
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(theme::BORDER_DIMMER)),
                    ..Default::default()
                })
                .width(Length::Fill),
        ];

        let is_selected = self.selected_entry == Some(idx);
        let base_style = if is_selected {
            theme::button_selected_row as fn(&Theme, button::Status) -> button::Style
        } else if is_dir {
            theme::button_dir_row
        } else if is_executable {
            theme::button_exec_row
        } else {
            theme::button_file_row
        };

        // If this is the hovered folder while dragging, highlight it heavily
        let is_hovered_target = self.hovered_row == Some(idx) 
            && self.dragging_item.is_some() 
            && self.dragging_item != Some(idx)
            && (is_dir || is_parent);

        let style = if is_hovered_target {
            theme::button_hovered_drop_target as fn(&Theme, button::Status) -> button::Style
        } else {
            base_style
        };

        let btn = button(row_with_border)
            .style(style)
            .padding(Padding::from([0, 0]))
            .width(Length::Fill)
            .on_press(Message::EntryClicked(idx));

        iced::widget::mouse_area(btn)
            .on_enter(Message::HoverRow(idx))
            .on_exit(Message::LeaveRow(idx))
            .into()
    }

    // ─── Keyboard subscription ───────────────────────────────────────────────
    fn subscription(&self) -> Subscription<Message> {
        let keyboard_sub = iced::keyboard::on_key_press(|key, modifiers| {
            match key {
                iced::keyboard::Key::Named(iced::keyboard::key::Named::F2) => {
                    Some(Message::StartRename)
                }
                iced::keyboard::Key::Named(iced::keyboard::key::Named::F5) => {
                    Some(Message::RefreshDir)
                }
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => {
                    Some(Message::CloseMenus)
                }
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete) => {
                    Some(Message::MoveToTrash)
                }
                iced::keyboard::Key::Character(ref c) if !modifiers.control() && modifiers.shift() => {
                    match c.as_str() {
                        "N" | "n" => Some(Message::NewFolder),
                        _ => None,
                    }
                }
                iced::keyboard::Key::Character(ref c) if modifiers.control() && !modifiers.shift() => {
                    match c.as_str() {
                        "h" => Some(Message::ToggleHidden),
                        "c" => Some(Message::CopySelected),
                        "x" => Some(Message::CutSelected),
                        "v" => Some(Message::PasteClipboard(None)),
                        "a" => Some(Message::SelectAll),
                        _ => None,
                    }
                }
                _ => None,
            }
        });

        // Global event listener for mouse drags
        let mouse_sub = iced::event::listen_with(|event, _status, _window| {
            // We want to capture mouse move and release globally always, otherwise we might miss drops.
            match event {
                iced::Event::Mouse(_) => Some(Message::GlobalEvent(event)),
                _ => None,
            }
        });

        let mut subs = vec![keyboard_sub, mouse_sub];
        
        // Always run the tick to dismiss notifications; also update transfer progress
        subs.push(iced::time::every(std::time::Duration::from_millis(100)).map(Message::TickTransfer));

        Subscription::batch(subs)
    }
}

fn dirs_or_root() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    PathBuf::from("/")
}

fn main() -> iced::Result {
    iced::application(
        OrbitalHud::title,
        OrbitalHud::update,
        OrbitalHud::view,
    )
    .theme(OrbitalHud::theme)
    .font(JETBRAINS_MONO_BYTES)
    .window_size(Size::new(1024.0, 768.0))
    .antialiasing(true)
    .subscription(OrbitalHud::subscription)
    .run_with(OrbitalHud::new)
}
