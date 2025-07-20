use passd::{configs::load_config, utils::logger::init_logger};

fn main() {
    let config = load_config().expect("Failed to load config");

    let _ = init_logger(&config.log_file, &config.log_level);
}
