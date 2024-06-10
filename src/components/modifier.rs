use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy::render::color::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Modifier {
    ColorModifier(Color),
    ScaleModifier(f32),
    ShinyModifier(f32),
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModifierName {
    pub name: String,
    pub modifier: Modifier,
}

pub fn get_color_from_hex(hex: &str) -> Color {
    let rgb = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&rgb[0..2], 16).unwrap();
    let g = u8::from_str_radix(&rgb[2..4], 16).unwrap();
    let b = u8::from_str_radix(&rgb[4..6], 16).unwrap();

    Color::rgb_u8(r, g, b)
}

#[derive(Debug, Serialize, Deserialize, Clone, DerefMut, Deref)]
pub struct Trie {
    #[deref]
    pub children: HashMap<char, Trie>,
    pub data: Option<ModifierName>,
}

#[derive(Resource)]
pub struct Dictionary {
    pub trie: Arc<Mutex<Trie>>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            trie: Arc::new(Mutex::new(Trie::new())),
        }
    }

    pub fn search(&mut self, word: &str) -> Option<ModifierName> {
        // check if the word is in the dictionary
        let search_res = self.trie.lock().unwrap().search(word);

        match search_res {
            Some(data) => Some(data),
            None => self.trie.lock().unwrap().import(word),
        }
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            children: HashMap::new(),
            data: None,
        }
    }

    pub fn insert(&mut self, word: &str, data: Modifier) -> ModifierName {
        let mut current = self;
        for c in word.chars() {
            current = current.children.entry(c).or_default();
        }
        let data = ModifierName {
            name: word.to_string(),
            modifier: data,
        };
        current.data = Some(data.clone());
        data
    }

    pub fn import(&mut self, word: &str) -> Option<ModifierName> {
        let mut path = Path::new("assets/dictionary").to_path_buf();

        for c in word.chars() {
            path.push(c.to_string());
        }

        path.push(word);

        let file_path = path.with_extension("ron");

        if file_path.exists() {
            let file = OpenOptions::new().read(true).open(file_path).unwrap();

            let data: Modifier = ron::de::from_reader(file).unwrap();

            return Some(self.insert(word, data));
        }

        None
    }

    pub fn search(&self, word: &str) -> Option<ModifierName> {
        let mut current = self;
        for c in word.chars() {
            if let Some(node) = current.children.get(&c) {
                current = node;
            } else {
                return None;
            }
        }

        current.data.clone()
    }

    pub fn export(&self, path: &Path) {
        for (c, node) in &self.children {
            let mut new_path = path.to_path_buf();

            // create if not exists

            new_path.push(c.to_string());
            std::fs::create_dir_all(&new_path).unwrap();

            if node.data.is_some() {
                let data = node.data.as_ref().unwrap();

                let content = ron::to_string(&data.modifier).unwrap();

                let mut file_path = new_path.clone();
                file_path.push(&data.name);

                // write the hex value to a file
                let mut file = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(file_path.with_extension("ron"))
                    .unwrap();

                file.write_all(content.as_bytes()).unwrap();
            }

            node.export(&new_path);
        }
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}
