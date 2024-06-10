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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Modifier {
    ColorModifier(Color),
    ScaleModifier(f32),
    RoughnessModifier(f32),
    MetallicModifier(f32),
    ReflectanceModifier(f32),
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModifierName {
    pub name: String,
    pub modifier: Vec<Modifier>,
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

    pub fn search(&mut self, word: &str) -> Vec<ModifierName> {
        // check if the word is in the dictionary
        let search_res = self.trie.lock().unwrap().search(word);

        match search_res {
            Some(data) => vec![data],
            None => match self.trie.lock().unwrap().import(word) {
                Some(data) => vec![data],
                None => vec![],
            },
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

        match current.data {
            Some(ref mut modifier_name) => {
                modifier_name.modifier.push(data);
                modifier_name.clone()
            }
            None => {
                let modifier_name = ModifierName {
                    name: word.to_string(),
                    modifier: vec![data],
                };
                current.data = Some(modifier_name.clone());
                modifier_name
            }
        }
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

            let data: Vec<Modifier> = ron::de::from_reader(file).unwrap();

            return Some(ModifierName {
                name: word.to_string(),
                modifier: data,
            });
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
                let mut data = node.data.as_ref().unwrap().clone();
                let mut file_path = new_path.clone();
                file_path.push(&data.name);

                if let Ok(file_read) = OpenOptions::new()
                    .read(true)
                    .open(file_path.with_extension("ron"))
                {
                    let existing_data: Vec<Modifier> =
                        ron::de::from_reader(&file_read).unwrap_or_default();

                    if !existing_data.is_empty() {
                        for modifier in &existing_data {
                            if !data.modifier.contains(modifier) {
                                data.modifier.push(modifier.clone());
                            }
                        }
                    }
                }

                let mut file_write = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(file_path.with_extension("ron"))
                    .unwrap();

                let content =
                    ron::ser::to_string_pretty(&data.modifier, Default::default()).unwrap();

                file_write.write_all(content.as_bytes()).unwrap();
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
