use scrb::components::modifier::{Modifier, Trie};

use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build a trie from the color names

    let rough = vec!["rough", "uneven", "jagged", "bumpy"]; // Added synonyms and related concepts
    let smooth = vec!["smooth", "even", "flat", "uniform"]; // Added antonyms and related concepts
    let metallic = vec!["iron", "steel", "metal", "aluminum", "golden"]; // Added materials that are often glossy

    let mut root = Trie::new();

    for rough_word in rough {
        root.insert(rough_word, Modifier::RoughnessModifier(1.0));
    }

    for smooth_word in smooth {
        root.insert(smooth_word, Modifier::RoughnessModifier(0.089));
    }

    for metallic_word in metallic {
        root.insert(metallic_word, Modifier::RoughnessModifier(0.089));
        root.insert(metallic_word, Modifier::MetallicModifier(1.0));
        root.insert(metallic_word, Modifier::ReflectanceModifier(0.5));
    }

    let root_dir_path = std::path::PathBuf::from_str("assets/dictionary")?;
    std::fs::create_dir_all(&root_dir_path)?;

    root.export(&root_dir_path);

    Ok(())
}
