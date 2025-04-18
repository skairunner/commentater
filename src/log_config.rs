// use log::LevelFilter;
use simplelog::ConfigBuilder;

pub fn default_log_config() -> simplelog::Config {
    ConfigBuilder::new()
        .set_time_offset_to_local()
        .unwrap()
        // .set_target_level(LevelFilter::Error)
        .add_filter_ignore_str("tracing::span")
        .build()
}
