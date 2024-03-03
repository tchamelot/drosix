use heck::ToSnakeCase;
use std::env;

const EXPORT_LIST: &[&str] = &["Pid", "AnglePid", "Odometry", "Angles", "DebugConfig", "PruSharedMem", "VolatileCell"];

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed=build.rs");

    let config = cbindgen::Config::from_root_or_default(&crate_dir);

    let mut builder = cbindgen::Builder::new().with_crate(crate_dir).with_config(config);

    for item in EXPORT_LIST {
        let renamed = item.to_snake_case();
        let cell = String::from("VolatileCell") + "_" + item;
        builder = builder.include_item(item).rename_item(item, &renamed.as_str()).rename_item(cell, renamed + "_t");
    }
    builder = builder.rename_item((String::from("VolatileCell") + "_u32").as_str(), "u32");

    builder.generate().expect("Unable to generate bindings").write_to_file("../pru/src/shared-memory.h");
}
