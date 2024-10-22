use config::Config;
use std::sync::LazyLock;

const CONFIG_FILE: &'static str = "drosix.toml";

pub static DROSIX_CONFIG: LazyLock<Config> = LazyLock::new(|| {
    Config::builder().add_source(config::File::with_name(CONFIG_FILE)).build().expect("Loading Drosix config")
});
