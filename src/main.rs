use chrono::{DateTime, Utc};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// ════════════════════════════════════════════════════════════ model ══

#[derive(Deserialize, Debug)]
struct MetadataFile {
    conversations: HashMap<String, ConversationMeta>,
}

#[derive(Deserialize, Debug)]
struct ConversationMeta {
    #[serde(default)]
    is_internal: bool,
    last_modified_time: Option<String>,
    summary: Option<Summary>,
}

#[derive(Deserialize, Debug)]
struct Summary {
    #[serde(default, rename = "Title")]
    title: Option<String>,
    #[serde(default, rename = "Preview")]
    preview: Option<String>,
    #[serde(default, rename = "NumSteps")]
    num_steps: Option<usize>,
    #[serde(default, rename = "WorkspaceURIs")]
    workspace_uris: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct MessageFile {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Clone, Debug)]
struct ConversationItem {
    user: String,
    id: String,
    path: String,
    title: String,
    steps: usize,
    workspaces: Vec<String>,
    date_str: String,
    sort_key: DateTime<Utc>,
}

// ════════════════════════════════════════════════════════════ users ══

struct UserPaths {
    name: String,
    cache_dir: PathBuf,
    brain_dir: PathBuf,
    readable: bool,
}

fn current_username() -> String {
    if let Ok(u) = std::env::var("USER") {
        return u;
    }
    if let Ok(u) = std::env::var("USERNAME") {
        return u;
    }
    if let Ok(h) = std::env::var("HOME")
        && let Some(name) = Path::new(&h).file_name().and_then(|n| n.to_str())
    {
        return name.to_string();
    }
    "local".to_string()
}

/// Zwraca ścieżkę domową usera (`/root` dla roota, `/home/<n>` reszta).
fn user_home_dir(name: &str) -> String {
    if name == "root" {
        "/root".to_string()
    } else {
        format!("/home/{name}")
    }
}

fn discover_users() -> Vec<UserPaths> {
    let mut users: Vec<UserPaths> = Vec::new();
    let current = current_username();

    // 1) bieżący użytkownik — zawsze czytelny
    if let Ok(home) = std::env::var("HOME") {
        let cache_dir = PathBuf::from(&home)
            .join(".agyw")
            .join("shared")
            .join("cache");
        let brain_dir = cache_dir
            .parent()
            .map(|p| p.join("brain"))
            .unwrap_or_else(|| PathBuf::from("brain"));
        users.push(UserPaths {
            name: current.clone(),
            cache_dir,
            brain_dir,
            readable: true,
        });
    }

    // 2) /root — dodaj zawsze (jak /home/*), z rozróżnieniem czytelny/nieczytelny
    if current != "root" {
        let cache_dir = PathBuf::from("/root/.agyw/shared/cache");
        let brain_dir = cache_dir
            .parent()
            .map(|p| p.join("brain"))
            .unwrap_or_else(|| cache_dir.clone());

        match fs::read_dir(&cache_dir) {
            Ok(_) => {
                users.push(UserPaths {
                    name: "root".into(),
                    cache_dir,
                    brain_dir,
                    readable: true,
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // /root nie używa agy — pomiń cicho
            }
            Err(_) => {
                // PermissionDenied na /root lub /root/.agyw — pokaż jako (?)
                users.push(UserPaths {
                    name: "root".into(),
                    cache_dir,
                    brain_dir,
                    readable: false,
                });
            }
        }
    }

    // 3) auto-skan /home/* i /Users/*
    for root in ["/home", "/Users"] {
        let Ok(entries) = fs::read_dir(root) else {
            continue;
        };
        for entry in entries.flatten() {
            let user_path = entry.path();
            if !user_path.is_dir() {
                continue;
            }
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            if name == current {
                continue;
            }

            let cache_dir = user_path.join(".agyw").join("shared").join("cache");
            let brain_dir = cache_dir
                .parent()
                .map(|p| p.join("brain"))
                .unwrap_or_else(|| cache_dir.clone());

            match fs::read_dir(&cache_dir) {
                Ok(_) => {
                    users.push(UserPaths {
                        name,
                        cache_dir,
                        brain_dir,
                        readable: true,
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // ten user nie używa agy — pomiń cicho
                    continue;
                }
                Err(_) => {
                    // PermissionDenied (lub inne) — dodaj jako nieczytelny
                    users.push(UserPaths {
                        name,
                        cache_dir,
                        brain_dir,
                        readable: false,
                    });
                }
            }
        }
    }

    // 4) env: AGY_EXTRA_USERS="ala:/path,bob:/path"
    if let Ok(extra) = std::env::var("AGY_EXTRA_USERS") {
        for entry in extra.split(',') {
            if let Some((name, path)) = entry.split_once(':') {
                let cache_dir = PathBuf::from(path.trim());
                let brain_dir = cache_dir
                    .parent()
                    .map(|p| p.join("brain"))
                    .unwrap_or_else(|| cache_dir.clone());
                let readable = fs::read_dir(&cache_dir).is_ok();
                users.push(UserPaths {
                    name: name.trim().to_string(),
                    cache_dir,
                    brain_dir,
                    readable,
                });
            }
        }
    }

    // 5) CLI: --user ala:/path
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--user" && i + 1 < args.len() {
            if let Some((name, path)) = args[i + 1].split_once(':') {
                let cache_dir = PathBuf::from(path);
                let brain_dir = cache_dir
                    .parent()
                    .map(|p| p.join("brain"))
                    .unwrap_or_else(|| cache_dir.clone());
                let readable = fs::read_dir(&cache_dir).is_ok();
                users.push(UserPaths {
                    name: name.trim().to_string(),
                    cache_dir,
                    brain_dir,
                    readable,
                });
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    let mut seen = HashSet::new();
    users.retain(|u| seen.insert(u.cache_dir.clone()));
    users
}

// ════════════════════════════════════════════════════════════ helpers ══

fn truncate_chars(s: &str, max_chars: usize) -> String {
    let mut it = s.chars();
    let mut out: String = it.by_ref().take(max_chars).collect();
    if it.next().is_some() {
        out.push('…');
    }
    out
}

fn short_id(id: &str, n: usize) -> String {
    id.chars().take(n).collect()
}

/// Bezpieczne escapowanie do `sh -c '...'` (pojedyncze cudzysłowy w środku).
fn shell_escape_single(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Dzieli string na fizyczne linie o zadanej szerokości — raz, na potrzeby wirtualizacji.
fn wrap_string(s: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut out = Vec::new();
    for line in s.split('\n') {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }
        let mut buf = String::new();
        let mut count = 0usize;
        for ch in line.chars() {
            if count >= width {
                out.push(std::mem::take(&mut buf));
                count = 0;
            }
            buf.push(ch);
            count += 1;
        }
        if !buf.is_empty() {
            out.push(buf);
        }
    }
    out
}

// ════════════════════════════════════════════════════════════ loader ══

fn load_conversations_for_user(user: &UserPaths) -> Vec<ConversationItem> {
    let last_convs_path = user.cache_dir.join("last_conversations.json");
    let metadata_path = user.cache_dir.join("conversation_metadata.json");

    let last_convs: HashMap<String, String> = fs::read_to_string(&last_convs_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let id_to_path: HashMap<String, String> = last_convs.into_iter().map(|(k, v)| (v, k)).collect();

    let meta_file: MetadataFile = fs::read_to_string(&metadata_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(MetadataFile {
            conversations: HashMap::new(),
        });

    let mut items: Vec<ConversationItem> = Vec::new();

    for (conv_id, data) in meta_file.conversations {
        if data.is_internal {
            continue;
        }

        let workspaces = data
            .summary
            .as_ref()
            .and_then(|s| s.workspace_uris.clone())
            .unwrap_or_default();

        let mut path = if let Some(ws) = workspaces.first() {
            ws.replace("file://", "")
        } else {
            id_to_path.get(&conv_id).cloned().unwrap_or_default()
        };

        if path.is_empty() {
            let potential_brain = user.brain_dir.join(&conv_id);
            path = if potential_brain.exists() {
                format!("[Brain ID: {}…]", short_id(&conv_id, 8))
            } else {
                "(brak folderu)".to_string()
            };
        }

        if path.contains("/tmp/agy-bridge") {
            continue;
        }

        let (title, steps) = match data.summary {
            Some(s) => {
                let t = s.title.filter(|v| !v.trim().is_empty());
                let p = s.preview.filter(|v| !v.trim().is_empty());
                let title = t.or(p).unwrap_or_else(|| "Brak tytułu".to_string());
                (title, s.num_steps.unwrap_or(0))
            }
            None => ("Brak podsumowania".to_string(), 0),
        };

        let mod_time_str = data.last_modified_time.unwrap_or_default();
        let (sort_key, date_str) = match DateTime::parse_from_rfc3339(&mod_time_str) {
            Ok(dt) => {
                let utc = dt.with_timezone(&Utc);
                (utc, utc.format("%Y-%m-%d %H:%M").to_string())
            }
            Err(_) => (DateTime::<Utc>::MIN_UTC, "Brak daty".to_string()),
        };

        items.push(ConversationItem {
            user: user.name.clone(),
            id: conv_id,
            path,
            title,
            steps,
            workspaces,
            date_str,
            sort_key,
        });
    }

    items
}

fn load_all_conversations(users: &[UserPaths]) -> Vec<ConversationItem> {
    let mut all = Vec::new();
    for user in users {
        all.extend(load_conversations_for_user(user));
    }
    all.sort_by_key(|a| std::cmp::Reverse(a.sort_key));
    all
}

fn build_details(item: &ConversationItem, brain_dir: &Path) -> String {
    let mut info = String::with_capacity(1024);

    let _ = write!(
        info,
        "Użytkownik: {}\nID: {}\nData: {}\nKroki: {}\n\nOryginalny Folder (Workspace):\n  {}\n\nTytuł:\n  {}\n",
        item.user, item.id, item.date_str, item.steps, item.path, item.title
    );

    if !item.workspaces.is_empty() {
        info.push_str("\nWszystkie Workspace URIs:\n");
        for ws in &item.workspaces {
            let _ = writeln!(info, "  - {}", ws);
        }
    }

    let target_brain = brain_dir.join(&item.id);
    if !target_brain.exists() {
        return info;
    }

    let messages_dir = target_brain.join(".system_generated").join("messages");
    if let Ok(entries) = fs::read_dir(&messages_dir) {
        let mut msg_files: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
            .collect();

        msg_files.sort_by(|a, b| {
            let ta = fs::metadata(a)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let tb = fs::metadata(b)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            tb.cmp(&ta)
        });

        if let Some(latest) = msg_files.first()
            && let Ok(s) = fs::read_to_string(latest)
            && let Ok(msg) = serde_json::from_str::<MessageFile>(&s)
            && let Some(txt) = msg.content.or(msg.text)
        {
            let preview = truncate_chars(&txt, 147);
            let _ = write!(info, "\nOstatnia wiadomość:\n  \"{}\"\n", preview);
        }
    }

    info.push_str("\n--- Pliki Raportów (.md) ---\n");
    if let Ok(entries) = fs::read_dir(&target_brain) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file()
                && p.extension().is_some_and(|e| e == "md")
                && let Some(fname) = p.file_name().and_then(|n| n.to_str())
            {
                let _ = writeln!(info, " 📄 {}", fname);
            }
        }
    }

    info
}

fn load_transcript_lines(item: &ConversationItem, brain_dir: &Path) -> Vec<String> {
    let mut lines = Vec::new();
    let transcript_path = brain_dir
        .join(&item.id)
        .join(".system_generated")
        .join("logs")
        .join("transcript.jsonl");

    if transcript_path.exists()
        && let Ok(content) = fs::read_to_string(&transcript_path)
    {
        for (idx, line) in content.lines().enumerate() {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                let role = val
                    .get("role")
                    .or_else(|| val.get("type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("WPIS");
                let text = val
                    .get("content")
                    .or_else(|| val.get("text"))
                    .or_else(|| val.get("message"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{:?}", val));

                lines.push(format!(
                    ">>> [Krok {} | Rola/Typ: {}]\n{}\n{}",
                    idx + 1,
                    role,
                    text,
                    "─".repeat(80)
                ));
            }
        }
    }

    if lines.is_empty() {
        lines.push("Brak wpisów w pliku transcript.jsonl dla tej sesji.".to_string());
    }
    lines
}

// ════════════════════════════════════════════════════════════════ app ══

#[derive(PartialEq)]
enum ActiveView {
    ListAndDetails,
    Transcript,
}

struct App {
    items: Vec<ConversationItem>,
    filtered_indices: Vec<usize>,
    list_labels: Vec<String>,
    list_state: ListState,

    search_query: String,
    is_searching: bool,

    active_view: ActiveView,
    selected_session: Option<ConversationItem>,

    user_brain_dirs: HashMap<String, PathBuf>,
    available_users: Vec<String>,
    unreadable: HashSet<String>,
    user_filter: Option<String>,
    show_user: bool,

    // panel szczegółów (wirtualizowany)
    details_raw: String,
    details_wrapped: Vec<String>,
    details_wrap_width: u16,
    details_scroll: usize,
    details_view_height: u16,
    details_cache_key: Option<(String, String)>,

    // transkrypt (wirtualizowany)
    transcript_lines: Vec<String>,
    transcript_wrapped: Vec<String>,
    transcript_wrap_width: u16,
    transcript_scroll: usize,
    transcript_view_height: u16,
}

impl App {
    fn new(items: Vec<ConversationItem>, users: &[UserPaths]) -> Self {
        let available_users: Vec<String> = {
            let mut names: Vec<String> = users.iter().map(|u| u.name.clone()).collect();
            names.sort();
            names.dedup();
            names
        };
        let show_user = available_users.len() > 1;

        let user_brain_dirs: HashMap<String, PathBuf> = users
            .iter()
            .map(|u| (u.name.clone(), u.brain_dir.clone()))
            .collect();

        let unreadable: HashSet<String> = users
            .iter()
            .filter(|u| !u.readable)
            .map(|u| u.name.clone())
            .collect();

        let mut app = Self {
            filtered_indices: Vec::new(),
            list_labels: Vec::new(),
            items,
            list_state: ListState::default(),
            search_query: String::new(),
            is_searching: false,
            active_view: ActiveView::ListAndDetails,
            selected_session: None,
            user_brain_dirs,
            available_users,
            unreadable,
            user_filter: None,
            show_user,
            details_raw: String::new(),
            details_wrapped: Vec::new(),
            details_wrap_width: 0,
            details_scroll: 0,
            details_view_height: 0,
            details_cache_key: None,
            transcript_lines: Vec::new(),
            transcript_wrapped: Vec::new(),
            transcript_wrap_width: 0,
            transcript_scroll: 0,
            transcript_view_height: 0,
        };
        app.rebuild_filter();
        app
    }

    fn current_item(&self) -> Option<&ConversationItem> {
        let idx = self.list_state.selected()?;
        let item_idx = *self.filtered_indices.get(idx)?;
        self.items.get(item_idx)
    }

    fn brain_dir_for(&self, item: &ConversationItem) -> &Path {
        self.user_brain_dirs
            .get(&item.user)
            .map(|p| p.as_path())
            .unwrap_or_else(|| Path::new("."))
    }

    fn rebuild_filter(&mut self) {
        let q = self.search_query.to_lowercase();
        self.filtered_indices.clear();
        self.list_labels.clear();
        for (i, item) in self.items.iter().enumerate() {
            if let Some(u) = &self.user_filter
                && &item.user != u
            {
                continue;
            }
            if !q.is_empty()
                && !item.title.to_lowercase().contains(&q)
                && !item.path.to_lowercase().contains(&q)
            {
                continue;
            }
            self.filtered_indices.push(i);
            let label = if self.show_user {
                format!(
                    "[{}] {} | {} (Kroki: {})",
                    item.user, item.date_str, item.title, item.steps
                )
            } else {
                format!("{} | {} (Kroki: {})", item.date_str, item.title, item.steps)
            };
            self.list_labels.push(label);
        }

        self.list_state.select(if self.filtered_indices.is_empty() {
            None
        } else {
            Some(0)
        });
        self.details_scroll = 0;
    }

    fn refresh_details(&mut self) {
        let key = self.current_item().map(|i| (i.user.clone(), i.id.clone()));
        if key == self.details_cache_key {
            return;
        }
        let text = match self.current_item() {
            Some(item) => build_details(item, self.brain_dir_for(item)),
            None => {
                let mut msg = String::from("Brak wyników wyszukiwania.");
                if let Some(u) = &self.user_filter
                    && self.unreadable.contains(u)
                {
                    let me = current_username();
                    let home = user_home_dir(u);
                    msg = format!(
                        "⚠ Brak dostępu do cache użytkownika '{u}'.\n\n\
                        Możliwe przyczyny:\n  • katalog {home}/.agyw jest niedostępny dla '{me}'\n\n\
                        Rozwiązania:\n\
                         1) Nadaj uprawnienia (jako root):\n\
                               setfacl -R -m u:{me}:rx {home}/.agyw\n\
                            lub:\n\
                               chmod o+rx {home}/.agyw {home}/.agyw/shared {home}/.agyw/shared/cache\n\
                               chmod o+r  {home}/.agyw/shared/cache/*.json\n\
                         2) Uruchom z sudo:\n\
                               sudo agy-menu\n\
                         3) Zdiagnozuj:\n\
                               agy-menu --diagnose\n"
                    );
                }
                msg
            }
        };
        self.details_raw = text;
        self.details_cache_key = key;
        self.details_wrapped.clear();
        self.details_wrap_width = 0;
        self.details_scroll = 0;
    }

    fn rewrap_details(&mut self, width: u16) {
        self.details_wrapped = wrap_string(&self.details_raw, (width as usize).max(20));
        self.details_wrap_width = width;
    }

    fn rewrap_transcript(&mut self, width: u16) {
        let joined = self.transcript_lines.join("\n");
        self.transcript_wrapped = wrap_string(&joined, (width as usize).max(20));
        self.transcript_wrap_width = width;
    }

    fn clear_filter(&mut self) {
        self.search_query.clear();
        self.rebuild_filter();
    }

    fn cycle_user_filter(&mut self) {
        let n = self.available_users.len();
        if n == 0 {
            return;
        }
        self.user_filter = match self.user_filter.as_ref() {
            None => Some(self.available_users[0].clone()),
            Some(current) => {
                let idx = self.available_users.iter().position(|u| u == current);
                match idx {
                    Some(i) if i + 1 < n => Some(self.available_users[i + 1].clone()),
                    _ => None,
                }
            }
        };
        self.rebuild_filter();
    }

    fn next(&mut self) {
        let len = self.filtered_indices.len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let len = self.filtered_indices.len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
    }

    fn go_to_start(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn go_to_end(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.list_state
                .select(Some(self.filtered_indices.len() - 1));
        }
    }

    fn load_transcript(&mut self) {
        let lines = match self.current_item() {
            Some(item) => load_transcript_lines(item, self.brain_dir_for(item)),
            None => Vec::new(),
        };
        self.transcript_lines = lines;
        self.transcript_wrapped.clear();
        self.transcript_wrap_width = 0;
        self.transcript_scroll = 0;
        self.active_view = ActiveView::Transcript;
    }

    fn transcript_to_start(&mut self) {
        self.transcript_scroll = 0;
    }

    fn transcript_to_end(&mut self) {
        let max = self
            .transcript_wrapped
            .len()
            .saturating_sub(self.transcript_view_height as usize);
        self.transcript_scroll = max;
    }

    fn details_to_start(&mut self) {
        self.details_scroll = 0;
    }

    fn details_to_end(&mut self) {
        let max = self
            .details_wrapped
            .len()
            .saturating_sub(self.details_view_height as usize);
        self.details_scroll = max;
    }
}

// ═══════════════════════════════════════════════════════════════ ui ══

fn draw(f: &mut Frame, app: &mut App) {
    app.refresh_details();

    let size = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(2)])
        .split(size);

    let user_label = match &app.user_filter {
        None => "wszyscy".to_string(),
        Some(u) if app.unreadable.contains(u) => format!("{} (?)", u),
        Some(u) => u.clone(),
    };

    match app.active_view {
        ActiveView::ListAndDetails => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(main_chunks[0]);

            // lewa: lista
            let title_text = if app.is_searching {
                format!(" Szukaj: {}█ ", app.search_query)
            } else if !app.search_query.is_empty() {
                format!(
                    " Lista (Filtr: {} | User: {}) [Ctrl+L by wyczyścić] ",
                    app.search_query, user_label
                )
            } else if app.show_user {
                format!(
                    " Historia rozmów AGY [User: {} | U - zmień] [/ - szukaj] ",
                    user_label
                )
            } else {
                " Historia rozmów AGY [/ - szukaj] ".to_string()
            };

            let list_items: Vec<ListItem> = app
                .list_labels
                .iter()
                .map(|s| ListItem::new(s.as_str()))
                .collect();

            let list = List::new(list_items)
                .block(Block::default().borders(Borders::ALL).title(title_text))
                .highlight_style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut app.list_state);

            // prawa: szczegóły (wirtualizowane)
            let area = chunks[1];
            let inner_w = area.width.saturating_sub(2);
            let inner_h = area.height.saturating_sub(2);

            if app.details_wrap_width != inner_w {
                app.rewrap_details(inner_w);
            }
            app.details_view_height = inner_h;

            let max_scroll = app.details_wrapped.len().saturating_sub(inner_h as usize);
            if app.details_scroll > max_scroll {
                app.details_scroll = max_scroll;
            }

            let start = app.details_scroll;
            let end = (start + inner_h as usize).min(app.details_wrapped.len());
            let visible = app.details_wrapped[start..end].join("\n");

            let details_widget = Paragraph::new(visible)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Szczegóły / Podgląd "),
                )
                .style(Style::default().fg(Color::Cyan));

            f.render_widget(details_widget, area);
        }
        ActiveView::Transcript => {
            let area = main_chunks[0];
            let inner_w = area.width.saturating_sub(2);
            let inner_h = area.height.saturating_sub(2);

            if app.transcript_wrap_width != inner_w {
                app.rewrap_transcript(inner_w);
            }
            app.transcript_view_height = inner_h;

            let max_scroll = app
                .transcript_wrapped
                .len()
                .saturating_sub(inner_h as usize);
            if app.transcript_scroll > max_scroll {
                app.transcript_scroll = max_scroll;
            }

            let start = app.transcript_scroll;
            let end = (start + inner_h as usize).min(app.transcript_wrapped.len());
            let visible = app.transcript_wrapped[start..end].join("\n");

            let widget = Paragraph::new(visible)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Pełny Transkrypt Sesji (transcript.jsonl) [B/T = powrót] "),
                )
                .style(Style::default().fg(Color::Yellow));

            f.render_widget(widget, area);
        }
    }

    let help_text = match app.active_view {
        ActiveView::ListAndDetails => {
            if app.is_searching {
                " Wpisz frazę | [Enter] zatwierdź | [Ctrl+U] wyczyść | [Ctrl+C] wyjdź "
            } else if app.show_user {
                " [↑/↓/j/k] Ruch | [Home/[/End/]] Lista | [Ctrl+Home/End] Szczegóły | [/] Szukaj | [U] User | [T] Transkrypt | [Enter] Wybierz | [Q] Wyjdź "
            } else {
                " [↑/↓/j/k] Ruch | [Home/[/End/]] Lista | [Ctrl+Home/End] Szczegóły | [/] Szukaj | [T] Transkrypt | [Enter] Wybierz | [Q] Wyjdź "
            }
        }
        ActiveView::Transcript => {
            " [↑/↓/j/k] Przewiń | [PgUp/PgDn] Szybko | [Home/[/End/]] Skok | [B/T] Powrót | [Q] Wyjdź "
        }
    };

    let help_bar =
        Paragraph::new(help_text).style(Style::default().fg(Color::Black).bg(Color::DarkGray));
    f.render_widget(help_bar, main_chunks[1]);
}

// ═════════════════════════════════════════════════════════════ launch ══

/// Uruchamia wybraną sesję. Dla innego użytkownika — przez `su`.
fn launch_session(item: &ConversationItem) -> Result<(), Box<dyn std::error::Error>> {
    let current_user = current_username();
    let target_path = PathBuf::from(&item.path);
    let path_exists = target_path.is_dir();

    if item.user != current_user {
        // ── INNY UŻYTKOWNIK → su - <user> -c "<cmd>" ──
        let mut inner = String::new();
        if path_exists {
            inner.push_str("cd ");
            inner.push_str(&shell_escape_single(&item.path));
            inner.push_str(" && ");
        }
        inner.push_str("agy --conversation ");
        inner.push_str(&shell_escape_single(&item.id));

        println!("\n╭─ Inny użytkownik ──────────────────────────");
        println!("│ Bieżący : {}", current_user);
        println!("│ Docelowy: {}", item.user);
        if path_exists {
            println!("│ Katalog : {}", item.path);
        } else {
            println!(
                "│ Katalog : (brak — uruchomię w $HOME użytkownika {})",
                item.user
            );
        }
        println!("╰────────────────────────────────────────────");
        println!(
            "\nWywołanie: su - {} -c {}",
            item.user,
            shell_escape_single(&inner)
        );
        println!("(może pojawić się prośba o hasło)\n");

        let status = Command::new("su")
            .arg("-")
            .arg(&item.user)
            .arg("-c")
            .arg(&inner)
            .status()?;

        if !status.success() {
            eprintln!("\nsu zakończył się kodem: {:?}", status.code());
            eprintln!("Spróbuj ręcznie:");
            eprintln!("  su - {} -c {}", item.user, shell_escape_single(&inner));
            eprintln!("  lub (jeśli masz sudo):");
            eprintln!(
                "  sudo -iu {} sh -c {}",
                item.user,
                shell_escape_single(&inner)
            );
        }
    } else {
        // ── BIEŻĄCY UŻYTKOWNIK → bezpośrednio agy ──
        let mut cmd = Command::new("agy");
        cmd.arg("--conversation").arg(&item.id);

        if path_exists {
            println!("\nKatalog roboczy: {}", item.path);
            cmd.current_dir(&target_path);
        } else {
            println!(
                "\nUwaga: Folder roboczy '{}' nie istnieje. Uruchamiam w bieżącym katalogu.",
                item.path
            );
        }
        println!("Uruchamianie konwersacji: {}\n", item.id);

        let status = cmd.status()?;
        if !status.success() {
            eprintln!("agy zakończył się kodem: {:?}", status.code());
        }
    }

    Ok(())
}

// ═════════════════════════════════════════════════════════════ main ══

type AppTerminal = Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let users = discover_users();

    if std::env::args().any(|a| a == "--diagnose") {
        let me = current_username();
        let total = users.len();
        let readable_count = users.iter().filter(|u| u.readable).count();
        let unreadable_count = total - readable_count;

        println!("Wykryci użytkownicy ({}):", total);
        for u in &users {
            let (status, reason) = if u.readable {
                ("OK  ", String::new())
            } else {
                ("BRAK", "  ← PermissionDenied".to_string())
            };
            let tag = if u.name == me { " (bieżący)" } else { "" };
            println!(
                "  [{}] {:<20}{} {}{}",
                status,
                u.name,
                tag,
                u.cache_dir.display(),
                reason
            );
        }
        println!(
            "\nRazem: {} użytkowników ({} czytelnych, {} nieczytelnych)",
            total, readable_count, unreadable_count
        );

        if unreadable_count > 0 {
            println!("\nAby uzyskać dostęp do nieczytelnych cache, nadaj ACL (jako root):");
            for u in users.iter().filter(|u| !u.readable) {
                let home = user_home_dir(&u.name);
                println!("  setfacl -R -m u:{}:rx {}/.agyw", me, home);
            }
            println!("\nAlternatywnie uruchom z sudo:");
            println!("  sudo agy-menu");
        }
        return Ok(());
    }

    if users.is_empty() {
        return Err("Nie znaleziono żadnego katalogu cache użytkownika.".into());
    }

    let items = load_all_conversations(&users);
    if items.is_empty() {
        println!("Brak zapisanych rozmów.");
        return Ok(());
    }

    let mut app = App::new(items, &users);

    let mut terminal: AppTerminal = ratatui::init();
    let run_result = run_loop(&mut terminal, &mut app);
    ratatui::restore();
    run_result?;

    if let Some(item) = app.selected_session {
        launch_session(&item)?;
    }

    Ok(())
}

fn run_loop(terminal: &mut AppTerminal, app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            let ctrl_c = ctrl && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'));
            if ctrl_c {
                break;
            }

            match app.active_view {
                ActiveView::ListAndDetails => {
                    if app.is_searching {
                        match key.code {
                            KeyCode::Enter => app.is_searching = false,
                            KeyCode::Char('u') | KeyCode::Char('U') if ctrl => {
                                app.search_query.clear();
                                app.rebuild_filter();
                            }
                            KeyCode::Backspace => {
                                app.search_query.pop();
                                app.rebuild_filter();
                            }
                            KeyCode::Char(c) => {
                                app.search_query.push(c);
                                app.rebuild_filter();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') => break,

                            KeyCode::Char('/') => app.is_searching = true,
                            KeyCode::Char('t') | KeyCode::Char('T') => app.load_transcript(),
                            KeyCode::Char('u') | KeyCode::Char('U') => app.cycle_user_filter(),

                            KeyCode::Char('l') | KeyCode::Char('L') if ctrl => {
                                app.clear_filter();
                            }

                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),

                            // Ctrl+Home / Ctrl+End → początek / koniec panelu szczegółów
                            KeyCode::Home if ctrl => app.details_to_start(),
                            KeyCode::End if ctrl => app.details_to_end(),

                            // Home / End → początek / koniec listy
                            KeyCode::Home | KeyCode::Char('[') => app.go_to_start(),
                            KeyCode::End | KeyCode::Char(']') => app.go_to_end(),

                            KeyCode::PageDown => {
                                app.details_scroll = app
                                    .details_scroll
                                    .saturating_add(app.details_view_height.max(1) as usize);
                            }
                            KeyCode::PageUp => {
                                app.details_scroll = app
                                    .details_scroll
                                    .saturating_sub(app.details_view_height.max(1) as usize);
                            }
                            KeyCode::Enter => {
                                if let Some(item) = app.current_item() {
                                    app.selected_session = Some(item.clone());
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                ActiveView::Transcript => match key.code {
                    KeyCode::Char('b') | KeyCode::Char('B') => {
                        app.active_view = ActiveView::ListAndDetails;
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        app.active_view = ActiveView::ListAndDetails;
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => break,

                    KeyCode::Down | KeyCode::Char('j') => {
                        app.transcript_scroll = app.transcript_scroll.saturating_add(1);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.transcript_scroll = app.transcript_scroll.saturating_sub(1);
                    }
                    KeyCode::PageDown => {
                        app.transcript_scroll = app
                            .transcript_scroll
                            .saturating_add(app.transcript_view_height.max(1) as usize);
                    }
                    KeyCode::PageUp => {
                        app.transcript_scroll = app
                            .transcript_scroll
                            .saturating_sub(app.transcript_view_height.max(1) as usize);
                    }
                    KeyCode::Home | KeyCode::Char('[') => app.transcript_to_start(),
                    KeyCode::End | KeyCode::Char(']') => app.transcript_to_end(),
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
