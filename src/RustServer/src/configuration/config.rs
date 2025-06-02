use config::Config;
use rayon::ThreadPoolBuilder;
use tracing::info;

#[derive(Debug, Default, serde::Deserialize, PartialEq)]
pub(crate) struct AppConfig {
    pub parsing_threads: usize,
    pub database_url: String,
    pub dmm_repo_url: String,
    pub dmm_local_path: String,
    pub imdb_minimum_score: f32,
}

pub fn load_config() -> anyhow::Result<AppConfig> {
    let config = Config::builder()
        .set_default("parsing_threads", 4)?
        .set_default(
            "dmm_repo_url",
            "https://github.com/debridmediamanager/hashlists.git",
        )?
        .set_default("dmm_local_path", "./data/dmm-hashlists")?
        .set_default("imdb_minimum_score", 0.85)?
        .add_source(config::Environment::with_prefix("ZILEAN"))
        .build()?;

    let app_config: AppConfig = config.try_deserialize()?;

    if app_config.parsing_threads == 0 {
        return Err(anyhow::anyhow!(
            "ZILEAN_PARSING_THREADS must be greater than 0"
        ));
    }

    if app_config.database_url.is_empty() {
        return Err(anyhow::anyhow!("ZILEAN_DATABASE_URL must be set"));
    }

    if app_config.imdb_minimum_score < 0.0 || app_config.imdb_minimum_score > 1.0 {
        return Err(anyhow::anyhow!(
            "ZILEAN_IMDB_MINIMUM_SCORE must be between 0.0 and 1.0"
        ));
    }

    info!("Using {} parsing threads.", &app_config.parsing_threads);
    info!(
        "Using IMDB minimum score: {}",
        &app_config.imdb_minimum_score
    );
    info!("Using DMM repo URL: {}", &app_config.dmm_repo_url);
    info!("Using DMM local path: {}", &app_config.dmm_local_path);

    ThreadPoolBuilder::new()
        .num_threads(app_config.parsing_threads)
        .build_global()
        .expect("Rayon pool already initialized");

    Ok(app_config)
}
