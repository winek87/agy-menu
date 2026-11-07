use crate::loader;
use crate::model::ConversationItem;
use ratatui::widgets::ListState;
use std::path::PathBuf;

#[derive(PartialEq)]
pub enum ActiveView {
    ListAndDetails,
    Transcript,
}

pub struct App {
    pub items: Vec<ConversationItem>,
    pub filtered_items: Vec<ConversationItem>,
    pub list_state: ListState,
    pub search_query: String,
    pub is_searching: bool,
    pub details_scroll: u16,
    pub transcript_scroll: u16,
    pub active_view: ActiveView,
    pub transcript_lines: Vec<String>,
    pub selected_session: Option<ConversationItem>,
    pub brain_dir: PathBuf,
    /// Cache treści panelu szczegółów (odświeżane tylko przy zmianie zaznaczenia).
    pub details_cache: String,
    details_cache_id: Option<String>,
}

impl App {
    pub fn new(items: Vec<ConversationItem>, brain_dir: PathBuf) -> Self {
        let mut app = Self {
            filtered_items: items.clone(),
            items,
            list_state: ListState::default(),
            search_query: String::new(),
            is_searching: false,
            details_scroll: 0,
            transcript_scroll: 0,
            active_view: ActiveView::ListAndDetails,
            transcript_lines: Vec::new(),
            selected_session: None,
            brain_dir,
            details_cache: String::new(),
            details_cache_id: None,
        };
        if !app.filtered_items.is_empty() {
            app.list_state.select(Some(0));
        }
        app
    }

    pub fn current_item(&self) -> Option<&ConversationItem> {
        let idx = self.list_state.selected()?;
        self.filtered_items.get(idx)
    }

    /// Wołane z `ui::draw` — przebudowuje `details_cache` tylko gdy zmieniło się ID.
    pub fn refresh_details(&mut self) {
        let current_id = self.current_item().map(|i| i.id.clone());
        if current_id == self.details_cache_id {
            return;
        }

        let text = match self.current_item() {
            Some(item) => loader::build_details(item, &self.brain_dir),
            None => "Brak wyników wyszukiwania".to_string(),
        };

        self.details_cache = text;
        self.details_cache_id = current_id;
    }

    pub fn update_filter(&mut self) {
        let q = self.search_query.to_lowercase();
        self.filtered_items = if q.is_empty() {
            self.items.clone()
        } else {
            self.items
                .iter()
                .filter(|item| {
                    item.title.to_lowercase().contains(&q)
                        || item.path.to_lowercase().contains(&q)
                })
                .cloned()
                .collect()
        };

        self.list_state.select(if self.filtered_items.is_empty() {
            None
        } else {
            Some(0)
        });
        self.details_scroll = 0;
    }

    pub fn next(&mut self) {
        let len = self.filtered_items.len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        self.list_state.select(Some(i));
        self.details_scroll = 0;
    }

    pub fn previous(&mut self) {
        let len = self.filtered_items.len();
        if len == 0 {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => len - 1,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
        self.details_scroll = 0;
    }

    pub fn go_to_start(&mut self) {
        if !self.filtered_items.is_empty() {
            self.list_state.select(Some(0));
            self.details_scroll = 0;
        }
    }

    pub fn go_to_end(&mut self) {
        if !self.filtered_items.is_empty() {
            self.list_state.select(Some(self.filtered_items.len() - 1));
            self.details_scroll = 0;
        }
    }

    pub fn load_transcript(&mut self) {
        let lines = match self.current_item() {
            Some(item) => loader::load_transcript_lines(item, &self.brain_dir),
            None => Vec::new(),
        };
        self.transcript_lines = lines;
        self.transcript_scroll = 0;
        self.active_view = ActiveView::Transcript;
    }
}
