use std::path::PathBuf;

#[path = "src/manifest.rs"]
mod manifest;

#[path = "src/schematic.rs"]
mod schematic;

fn main() {
    println!("cargo::rerun-if-changed=src/manifest.rs,src/schematic.rs");

    let schema_dir: PathBuf = "schema".into();
    std::fs::create_dir_all(&schema_dir).expect("Failed to create schema directory");

    // Generate the schema for the manifest
    generate_schema::<manifest::Manifest>(&schema_dir.join("commands.json"));

    // Generate the schema for the schematic
    generate_schema::<schematic::Soc>(&schema_dir.join("schematic-soc.json"));
    generate_schema::<schematic::Board>(&schema_dir.join("schematic-board.json"));
    generate_schema::<schematic::App>(&schema_dir.join("schematic-app.json"));
}

fn generate_schema<T: schemars::JsonSchema>(path: &PathBuf) {
    let schema = schemars::schema_for!(T);
    let schema_json = serde_json::to_string_pretty(&schema).expect("Failed to serialize schema");
    std::fs::write(path, schema_json).expect("Failed to write schema to file");
}
