use core::fmt;
use std::path::PathBuf;

use leveldb::{LdbIterator, Options};
use miette::IntoDiagnostic;
use ratatui::{
    style::{Color, Style},
    text::Line,
    widgets::{ListState, ScrollbarState},
};
use rustc_serialize::json::ToJson as _;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Focus {
    Keys,
    Value,
}

impl Focus {
    pub fn switch(&mut self) {
        match self {
            Focus::Keys => *self = Self::Value,
            Focus::Value => *self = Self::Keys,
        }
    }
}

pub struct App {
    pub(crate) path: PathBuf,
    pub(crate) keys_widget_state: ListState,
    pub(crate) value_scroll: usize,
    pub(crate) value_scroll_state: ScrollbarState,
    pub(crate) focus: Focus,
    pub(crate) db: leveldb::DB,

    // Preloaded keys for easier interaction.
    pub(crate) keys: Vec<String>,
}

#[derive(Default)]
pub enum ContentType {
    Json,
    Hex,
    Cbor,
    #[default]
    Undefined,
}

impl ContentType {
    pub fn to_line(&self) -> Line {
        match self {
            ContentType::Json => Line::from("JSON").style(Style::new().bg(Color::Yellow)),
            ContentType::Cbor => Line::from("CBOR").style(Style::new().bg(Color::Blue)),
            ContentType::Hex => Line::from("HEX").style(Style::new().bg(Color::Gray)),
            ContentType::Undefined => Line::from(""),
        }
        .left_aligned()
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Json => write!(f, "JSON"),
            ContentType::Hex => write!(f, "HEX"),
            ContentType::Cbor => write!(f, "CBOR"),
            ContentType::Undefined => write!(f, ""),
        }
    }
}

impl App {
    pub fn from_db_path(path: PathBuf) -> miette::Result<Self> {
        let mut db = leveldb::DB::open(&path, Options::default()).into_diagnostic()?;
        let mut db_iter = db.new_iter().into_diagnostic()?;

        let mut keys = Vec::new();
        while let Some((key, _)) = db_iter.next() {
            keys.push(String::from_utf8(key.clone()).unwrap_or_else(|_| hex::encode(&key)));
        }

        Ok(Self {
            path,
            keys_widget_state: ListState::default(),
            value_scroll: 0,
            value_scroll_state: ScrollbarState::default(),
            focus: Focus::Keys,
            db,
            keys,
        })
    }

    pub fn get_value_by_key_idx(&mut self) -> Option<(ContentType, String)> {
        let value = self
            .keys_widget_state
            .selected()
            .and_then(|idx| self.keys.get(idx))
            .and_then(|key| self.db.get(key.as_bytes()))?;

        Some(
            String::from_utf8(value.clone())
                .into_diagnostic()
                .and_then(|val_str| json::parse(val_str.as_str()).into_diagnostic())
                .map(|val_json| (ContentType::Json, val_json.pretty(2)))
                .or_else(|_| -> miette::Result<_> {
                    let mut decoder = cbor::Decoder::from_bytes(value.as_slice());
                    let cbor = decoder.items().next().unwrap().into_diagnostic()?;

                    Ok((ContentType::Cbor, cbor.to_json().pretty().to_string()))
                })
                .unwrap_or_else(|_| (ContentType::Hex, hex::encode(value))),
        )
    }
}
