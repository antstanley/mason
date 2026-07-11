use crate::cache::Caches;
use crate::config::Config;
use crate::http::Http;

pub struct AppState {
    pub config: Config,
    pub http: Http,
    pub caches: Caches,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            http: Http::new(),
            caches: Caches::new(),
        }
    }
}
