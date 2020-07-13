use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub atom: AtomConf,
    pub topics: Vec<TopicConf>,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let mut settings = config::Config::default();
        settings
            // Add in `./Settings.toml`
            .merge(config::File::with_name("Settings"))
            .unwrap()
            .merge(config::Environment::with_prefix("ATOM"))?;
        let s: Settings = settings.try_into()?;

        Ok(s)
    }

    pub fn contains_topic(&self, topic: &str) -> bool {
        if let Some(_) = self.get_topic(topic) {
            return true;
        }

        false
    }

    pub fn get_topic(&self, topic: &str) -> Option<TopicConf> {
        for item in &self.topics {
            if item.topic == topic {
                return Some(item.clone());
            }
        }

        None
    }

    pub fn get_webhook_by_topic(&self, topic: &str) -> Option<String> {
        if let Some(item) = self.get_topic(topic) {
            match item.webhook {
                Some(v) => return Some(v),
                None => return None,
            };
        }

        None
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AtomConf {
    pub db_url: String,
    pub prs_base_url: String,
    pub bind_address: String,
    pub sentry_dsn: Option<String>,
    pub xml_output_dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicConf {
    pub topic: String,
    pub webhook: Option<String>,
    pub encryption_key: String,
    pub iv_prefix: String,
}
