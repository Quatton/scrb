use scrb::components::color::{ColorName, Trie};
use serde_json::Result;
use std::{fs::OpenOptions, str::FromStr};

fn main() -> Result<()> {
    // load colornames.json

    let file = OpenOptions::new()
        .read(true)
        .open("colornames.json")
        .unwrap();

    let data: Vec<ColorName> = serde_json::from_reader(file)?;

    // build a trie from the color names
    let mut root = Trie::new();

    for color in data {
        let name = color.name.clone().to_lowercase();
        let name_split = name.split_whitespace();

        if name_split.count() > 1 {
            continue;
        }

        root.insert(&name, color.hex);
    }

    // export the trie to a file nested with folders

    let root_dir_path = std::path::PathBuf::from_str("assets/dictionary").unwrap();
    std::fs::create_dir_all(&root_dir_path).unwrap();

    root.export(&root_dir_path);

    Ok(())
}
