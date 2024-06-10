use scrb::components::modifier::{Modifier, Trie};

use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build a trie from the color names

    let scales = [
        vec!["rough", "uneven", "jagged", "bumpy"], // Added synonyms and related concepts
        vec!["smooth", "even", "flat", "uniform"],  // Added antonyms and related concepts
        vec!["iron", "steel", "metal", "aluminum", "gold"], // Added materials that are often glossy
    ];

    let mut root = Trie::new();

    for (idx, roughness) in scales.iter().enumerate() {
        // rough is 1 and iron is 0.089, we will interpolate between them

        let calibrated_roughness = 1.0 - (0.911_f32 * idx as f32 / (scales.len() - 1) as f32);

        for roughness_word in roughness {
            root.insert(
                roughness_word,
                Modifier::ShinyModifier(calibrated_roughness),
            );
        }
    }

    // export the trie to a file nested with folders

    let root_dir_path = std::path::PathBuf::from_str("assets/dictionary")?;
    std::fs::create_dir_all(&root_dir_path)?;

    root.export(&root_dir_path);

    Ok(())
}
