use std::{collections::HashMap, fs::OpenOptions, io::Write, path::Path};

use bevy::prelude::*;
use bevy::render::color::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColorName {
    pub name: String,
    pub hex: String,
}

impl ColorName {
    pub fn get_color_from_hex(hex: &str) -> Color {
        let rgb = hex.trim_start_matches('#');

        let r = u8::from_str_radix(&rgb[0..2], 16).unwrap();
        let g = u8::from_str_radix(&rgb[2..4], 16).unwrap();
        let b = u8::from_str_radix(&rgb[4..6], 16).unwrap();

        Color::rgb_u8(r, g, b)
    }

    pub fn get_hex_from_color(color: &Color) -> String {
        let r = (color.r() * 255.0) as u8;
        let g = (color.g() * 255.0) as u8;
        let b = (color.b() * 255.0) as u8;

        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

pub struct Trie {
    pub children: HashMap<char, Trie>,
    pub data: Option<ColorName>,
    pub is_end: bool,
}

#[derive(Resource)]
pub struct ColorDictionary {
    pub trie: Trie,
}

impl ColorDictionary {
    pub fn new() -> Self {
        Self { trie: Trie::new() }
    }

    pub fn search(&mut self, word: &str) -> Option<Color> {
        if let Some(color_name) = self.trie.search(word) {
            let color = ColorName::get_color_from_hex(&color_name.hex);
            return Some(color);
        }

        // if the color name is not found, try to import from the dictionary
        let mut path = Path::new("assets/dictionary").to_path_buf();

        for c in word.chars() {
            path.push(c.to_string());
        }

        path.push(word);

        let file_path = path.with_extension("ron");

        if file_path.exists() {
            let file = OpenOptions::new().read(true).open(file_path).unwrap();

            let color: Color = ron::de::from_reader(file).unwrap();

            self.trie
                .insert(word, ColorName::get_hex_from_color(&color));

            return Some(color);
        }

        None
    }
}

impl Default for ColorDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            children: HashMap::new(),
            data: None,
            is_end: false,
        }
    }

    pub fn insert(&mut self, word: &str, hex: String) {
        let mut current = self;
        for c in word.chars() {
            current = current.children.entry(c).or_default();
        }
        current.is_end = true;
        current.data = Some(ColorName {
            name: word.to_string(),
            hex,
        });
    }

    pub fn search(&self, word: &str) -> Option<&ColorName> {
        let current = {
            let mut current = self;
            for c in word.chars() {
                if let Some(node) = current.children.get(&c) {
                    current = node;
                } else {
                    return None;
                }
            }
            Some(current)
        };
        if let Some(current) = current {
            if current.is_end {
                return current.data.as_ref();
            }
        }
        None
    }

    pub fn export(&self, path: &Path) {
        for (c, node) in &self.children {
            let mut new_path = path.to_path_buf();

            // create if not exists

            new_path.push(c.to_string());
            std::fs::create_dir_all(&new_path).unwrap();

            if node.is_end {
                let ColorName { hex, name } = node.data.as_ref().unwrap();

                let color = ColorName::get_color_from_hex(hex);
                let content = ron::to_string(&color).unwrap();

                let mut file_path = new_path.clone();
                file_path.push(name);

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
