use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct MetadataFile {
    pub conversations: HashMap<String, ConversationMeta>,
}

#[derive(Deserialize, Debug)]
pub struct ConversationMeta {
    #[serde(default)]
    pub is_internal: bool,
    pub last_modified_time: Option<String>,
    pub summary: Option<Summary>,
}

#[derive(Deserialize, Debug)]
pub struct Summary {
    #[serde(default, rename = "Title")]
    pub title: Option<String>,
    #[serde(default, rename = "Preview")]
    pub preview: Option<String>,
    #[serde(default, rename = "NumSteps")]
    pub num_steps: Option<usize>,
    #[serde(default, rename = "WorkspaceURIs")]
    pub workspace_uris: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct MessageFile {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ConversationItem {
    pub id: String,
    pub path: String,
    pub title: String,
    pub steps: usize,
    pub workspaces: Vec<String>,
    pub date_str: String,
    pub sort_key: DateTime<Utc>,
}
