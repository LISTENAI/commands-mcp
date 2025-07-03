#[path = "src/manifest.rs"]
mod manifest;

fn main() {
    println!("cargo:rerun-if-changed=src/manifest.rs");

    // Generate the schema for the manifest
    let schema = schemars::schema_for!(manifest::Manifest);
    let schema_json = serde_json::to_string_pretty(&schema).expect("Failed to serialize schema");
    std::fs::write("commands.schema.json", schema_json).expect("Failed to write schema to file");
}
