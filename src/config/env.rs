use std::env;


pub struct Config {
    pub log_level: String,
    pub database_url: String,
    pub jwt_secret: String,

    pub http_server_max_connexion: usize,
    pub http_server_num_worker: usize,
    pub http_server_hostname: String,

    pub http_listen_ip: String,
    pub http_listen_port: usize,

    pub prometheus_metrics_path: String,
    pub prometheus_namespace: String,
}


fn env_or_panic(name: String) -> String {
    let message = format!(
        "Env var {} must be set",
        name.as_str()
    );

    env::var(name)
        .unwrap_or_else(|_| { panic!("{}", message) })
}

fn env_or_string(name: String, default : String) -> String {
    env::var(name).unwrap_or(default)
}

fn env_or_int(name: String, default : String) -> usize {
    env::var(name)
        .unwrap_or(
            default
        )
        .to_string()
        .parse::<usize>()
        .unwrap()
}

pub fn load_config_from_env () -> Config {

    Config {
        log_level: env_or_string("RUST_LOG".to_string(), "DEBUG".to_string()),
        database_url: env_or_panic("DATABASE_URL".to_string()),
        jwt_secret: env_or_panic("JWT_SECRET".to_string()),

        http_server_max_connexion: env_or_int("HTTP_SERVER_MAX_CONNEXION".to_string(), "5".to_string()),
        http_server_num_worker: env_or_int("HTTP_SERVER_NUM_WORKERS".to_string(), "5".to_string()),

        http_server_hostname: env_or_string("HTTP_SERVER_HOSTNAME".to_string(), "localhost".to_string()),
        http_listen_ip: env_or_string("HTTP_LISTEN_IP".to_string(), "0.0.0.0".to_string()),
        http_listen_port: env_or_int("HTTP_LISTEN_PORT".to_string(), "8080".to_string()),

        prometheus_metrics_path: env_or_string("PROMETHEUS_METRICS_PATH".to_string(), "/metrics".to_string()),
        prometheus_namespace: env_or_string("PROMETHEUS_NAMESPACE".to_string(), "rust-playground".to_string()),
    }
}
