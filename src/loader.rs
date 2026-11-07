use crate::model::{ConversationItem, MessageFile, MetadataFile};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Paths {
    pub cache_dir: PathBuf,
    pub brain_dir: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME")?;
        let cache_dir = PathBuf::from(home)
            .join(".agyw")
            .join("shared")
            .join("cache");
        let brain_dir = cache_dir
            .parent()
            .ok_or("cache_dir has no parent")?
            .join("brain");
        Ok(Self { cache_dir, brain_dir })
    }
}

/// Bezpieczne cięcie po znakach (NIE po bajtach) — nie panikuje na UTF-8.
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

pub fn load_conversations(paths: &Paths) -> Vec<ConversationItem> {
    let last_convs_path = paths.cache_dir.join("last_conversations.json");
    let metadata_path = paths.cache_dir.join("conversation_metadata.json");

    let last_convs: HashMap<String, String> = fs::read_to_string(&last_convs_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    let id_to_path: HashMap<String, String> = last_convs
        .into_iter()
        .map(|(k, v)| (v, k))
        .collect();

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
            let potential_brain = paths.brain_dir.join(&conv_id);
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
            id: conv_id,
            path,
            title,
            steps,
            workspaces,
            date_str,
            sort_key,
        });
    }

    items.sort_by(|a, b| b.sort_key.cmp(&a.sort_key));
    items
}

/// Buduje treść panelu szczegółów — wołane raz na zmianę zaznaczenia (cache w App).
pub fn build_details(item: &ConversationItem, brain_dir: &Path) -> String {
    let mut info = String::with_capacity(1024);

    let _ = write!(
        info,
        "ID: {}\nData: {}\nKroki: {}\n\nOryginalny Folder (Workspace):\n  {}\n\nTytuł:\n  {}\n",
        item.id, item.date_str, item.steps, item.path, item.title
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

    // Ostatnia wiadomość
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

        if let Some(latest) = msg_files.first() {
            if let Ok(s) = fs::read_to_string(latest) {
                if let Ok(msg) = serde_json::from_str::<MessageFile>(&s) {
                    if let Some(txt) = msg.content.or(msg.text) {
                        let preview = truncate_chars(&txt, 147);
                        let _ = write!(info, "\nOstatnia wiadomość:\n  \"{}\"\n", preview);
                    }
                }
            }
        }
    }

    // Pliki .md
    info.push_str("\n--- Pliki Raportów (.md) ---\n");
    if let Ok(entries) = fs::read_dir(&target_brain) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() && p.extension().is_some_and(|e| e == "md") {
                if let Some(fname) = p.file_name().and_then(|n| n.to_str()) {
                    let _ = writeln!(info, " 📄 {}", fname);
                }
            }
        }
    }

    info
}

pub fn load_transcript_lines(item: &ConversationItem, brain_dir: &Path) -> Vec<String> {
    let mut lines = Vec::new();
    let transcript_path = brain_dir
        .join(&item.id)
        .join(".system_generated")
        .join("logs")
        .join("transcript.jsonl");

    if transcript_path.exists() {
        if let Ok(content) = fs::read_to_string(&transcript_path) {
            for (idx, line) in content.lines().enumerate() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                    let role = val
                        .get("role")
                        .or(val.get("type"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("WPIS");
                    let text = val
                        .get("content")
                        .or(val.get("text"))
                        .or(val.get("message"))
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
    }

    if lines.is_empty() {
        lines.push("Brak wpisów w pliku transcript.jsonl dla tej sesji.".to_string());
    }

    lines
}
