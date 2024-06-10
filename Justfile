run:
  cargo run --features bevy/dynamic_linking

build_trie:
  cargo run --bin build_color_trie
  cargo run --bin build_roughness_trie
  cargo run --bin build_scale_trie