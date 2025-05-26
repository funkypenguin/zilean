use config::Config;
use rayon::ThreadPoolBuilder;
use tracing::info;

#[derive(Debug, Default, serde::Deserialize, PartialEq, Eq)]
pub(crate) struct AppConfig {
    pub parsing_threads: usize,
    pub database_url: String
}

pub fn load_config() -> anyhow::Result<AppConfig> {
    let config = Config::builder()
        .set_default("parsing_threads", 4)?
        .add_source(config::Environment::with_prefix("ZILEAN"))
        .build()?;

    let app_config: AppConfig = config.try_deserialize()?;

    if app_config.parsing_threads == 0 {
        return Err(anyhow::anyhow!("ZILEAN_PARSING_THREADS must be greater than 0"));
    }

    if app_config.database_url.is_empty() {
        return Err(anyhow::anyhow!("ZILEAN_DATABASE_URL must be set"));
    }

    info!("Using {} parsing threads.", &app_config.parsing_threads);

    ThreadPoolBuilder::new()
        .num_threads(app_config.parsing_threads.clone())
        .build_global()
        .expect("Rayon pool already initialized");

    Ok(app_config)
}