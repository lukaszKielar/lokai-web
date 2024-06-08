use std::env;

use once_cell::sync::Lazy;

fn get_env_var<'a, 'b>(env_var: &'a str, default: &'b str) -> String {
    env::var(env_var).unwrap_or(default.to_string())
}

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub ollama_url: String,
    pub lokai_default_llm_model: String,
    pub lokai_host: String,
    pub lokai_port: String,
}

impl Config {
    fn new() -> Self {
        Self {
            database_url: get_env_var("DATABASE_URL", "sqlite://db.sqlite3"),
            ollama_url: get_env_var("OLLAMA_URL", "http://host.docker.internal:11434"),
            lokai_default_llm_model: get_env_var("LOKAI_DEFAULT_LLM_MODEL", "phi3:3.8b"),
            lokai_host: get_env_var("LOKAI_HOST", "0.0.0.0"),
            lokai_port: get_env_var("LOKAI_PORT", "3000"),
        }
    }

    pub fn lokai_url(&self) -> String {
        format!("{}:{}", self.lokai_host, self.lokai_port)
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::new());
