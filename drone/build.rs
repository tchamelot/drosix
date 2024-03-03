use heck::ToSnakeCase;
use std::env;

const EXPORT_LIST: &[&str] = &["Pid", "AnglePid", "Odometry", "Angles", "DebugConfig", "PruSharedMem", "VolatileCell"];

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    dbg!(&crate_dir);
    println!("cargo:rerun-if-changed=build.rs");

    // TODO use with_config to rename enum to qualified screaming snake case
    let mut builder = cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_tab_width(2)
        .with_style(cbindgen::Style::Tag)
        .with_pragma_once(true)
        .with_documentation(false)
        .with_parse_deps(true)
        .with_parse_include(&["prusst"]);

    for item in EXPORT_LIST {
        let renamed = item.to_snake_case();
        let cell = String::from("VolatileCell") + "_" + item;
        builder = builder.include_item(item).rename_item(item, &renamed.as_str()).rename_item(cell, renamed + "_t");
    }
    builder = builder.rename_item((String::from("VolatileCell") + "_u32").as_str(), "u32");

    builder.generate().expect("Unable to generate bindings").write_to_file("../pru/src/shared-memory.h");
}
