use scrb::components::modifier::{Modifier, Trie};

use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // build a trie from the color names

    let scales = [
        vec!["molecular", "atomic", "subatomic", "nano"], // Added smaller scale concepts
        vec!["tiny", "minuscule", "petite", "microscopic"], // Added synonyms and related concepts
        vec!["little", "slight", "minor", "diminutive"],  // Added synonyms to enhance detail
        vec!["small", "compact", "miniature", "modest"], // Expanded with synonyms and related sizes
        vec!["medium", "moderate", "medium-sized", "average"], // Clarified the middle of the scale
        vec!["big", "large", "substantial", "considerable"], // Added words that reflect bigger size
        vec!["huge", "massive", "enormous", "immense"],  // Enhanced with synonyms for very large
        vec!["giant", "titanic", "monstrous", "towering"], // Added dramatic size descriptors
        vec!["colossal", "mammoth", "gargantuan", "monumental"], // Synonyms for extremely large
        vec!["cosmic", "astronomical", "galactic", "stellar"], // Added space-related terms for vastness
        vec!["universal", "multiversal", "infinite", "boundless"], // Expanded to imply beyond a single universe
    ];
    let mut root = Trie::new();

    for (idx, scale) in scales.iter().enumerate() {
        // 4 is the base scale then 10% for each level
        let calibrated_scale = 1.5_f32.powi(idx as i32 - 2);

        for scale_word in scale {
            root.insert(scale_word, Modifier::ScaleModifier(calibrated_scale));
        }
    }

    // export the trie to a file nested with folders

    let root_dir_path = std::path::PathBuf::from_str("assets/dictionary")?;
    std::fs::create_dir_all(&root_dir_path)?;

    root.export(&root_dir_path);

    Ok(())
}
