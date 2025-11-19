use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(author, version, about = "Prints its configuration to STDOUT.")]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, env = "CONF_FILE", default_value = "config.toml")]
    conf: PathBuf,

    /// Enables debug mode
    #[arg(short, long)]
    debug: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default)]
    mode: ModeConfig,
    #[serde(default)]
    server: ServerConfig,
    #[serde(default)]
    db: DatabaseConfig,
    #[serde(default)]
    log: LogConfig,
    #[serde(default)]
    background: BackgroundConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: ModeConfig::default(),
            server: ServerConfig::default(),
            db: DatabaseConfig::default(),
            log: LogConfig::default(),
            background: BackgroundConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ModeConfig {
    #[serde(default = "default_debug")]
    debug: bool,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            debug: default_debug(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerConfig {
    #[serde(default = "default_external_url")]
    external_url: String,
    #[serde(default = "default_http_port")]
    http_port: u16,
    #[serde(default = "default_grpc_port")]
    grpc_port: u16,
    #[serde(default = "default_healthz_port")]
    healthz_port: u16,
    #[serde(default = "default_metrics_port")]
    metrics_port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            external_url: default_external_url(),
            http_port: default_http_port(),
            grpc_port: default_grpc_port(),
            healthz_port: default_healthz_port(),
            metrics_port: default_metrics_port(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConfig {
    #[serde(default)]
    mysql: MysqlConfig,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            mysql: MysqlConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MysqlConfig {
    #[serde(default = "default_mysql_host")]
    host: String,
    #[serde(default = "default_mysql_port")]
    port: u16,
    #[serde(default = "default_mysql_database")]
    database: String,
    #[serde(default = "default_mysql_user")]
    user: String,
    #[serde(default = "default_mysql_pass")]
    pass: String,
    #[serde(default)]
    connections: ConnectionLimits,
}

impl Default for MysqlConfig {
    fn default() -> Self {
        Self {
            host: default_mysql_host(),
            port: default_mysql_port(),
            database: default_mysql_database(),
            user: default_mysql_user(),
            pass: default_mysql_pass(),
            connections: ConnectionLimits::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ConnectionLimits {
    #[serde(default = "default_connections_max_idle")]
    max_idle: u32,
    #[serde(default = "default_connections_max_open")]
    max_open: u32,
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        Self {
            max_idle: default_connections_max_idle(),
            max_open: default_connections_max_open(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LogConfig {
    #[serde(default)]
    app: LogAppConfig,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            app: LogAppConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct LogAppConfig {
    #[serde(default = "default_log_level")]
    level: String,
}

impl Default for LogAppConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BackgroundConfig {
    #[serde(default)]
    watchdog: WatchdogConfig,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            watchdog: WatchdogConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WatchdogConfig {
    #[serde(default = "default_watchdog_period", with = "humantime_serde")]
    period: Duration,
    #[serde(default = "default_watchdog_limit")]
    limit: u64,
    #[serde(default = "default_watchdog_lock_timeout", with = "humantime_serde")]
    lock_timeout: Duration,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            period: default_watchdog_period(),
            limit: default_watchdog_limit(),
            lock_timeout: default_watchdog_lock_timeout(),
        }
    }
}

fn default_debug() -> bool {
    false
}

fn default_external_url() -> String {
    "http://127.0.0.1".to_string()
}

fn default_http_port() -> u16 {
    8081
}

fn default_grpc_port() -> u16 {
    8082
}

fn default_healthz_port() -> u16 {
    10025
}

fn default_metrics_port() -> u16 {
    9199
}

fn default_mysql_host() -> String {
    "127.0.0.1".to_string()
}

fn default_mysql_port() -> u16 {
    3306
}

fn default_mysql_database() -> String {
    "default".to_string()
}

fn default_mysql_user() -> String {
    "root".to_string()
}

fn default_mysql_pass() -> String {
    String::new()
}

fn default_connections_max_idle() -> u32 {
    30
}

fn default_connections_max_open() -> u32 {
    30
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_watchdog_period() -> Duration {
    Duration::from_secs(5)
}

fn default_watchdog_limit() -> u64 {
    10
}

fn default_watchdog_lock_timeout() -> Duration {
    Duration::from_secs(4)
}

fn load_config(cli: &Cli) -> Result<AppConfig> {
    let builder = Config::builder()
        .set_default("mode.debug", default_debug())?
        .set_default("server.external_url", default_external_url())?
        .set_default("server.http_port", default_http_port())?
        .set_default("server.grpc_port", default_grpc_port())?
        .set_default("server.healthz_port", default_healthz_port())?
        .set_default("server.metrics_port", default_metrics_port())?
        .set_default("db.mysql.host", default_mysql_host())?
        .set_default("db.mysql.port", default_mysql_port())?
        .set_default("db.mysql.database", default_mysql_database())?
        .set_default("db.mysql.user", default_mysql_user())?
        .set_default("db.mysql.pass", default_mysql_pass())?
        .set_default(
            "db.mysql.connections.max_idle",
            default_connections_max_idle(),
        )?
        .set_default(
            "db.mysql.connections.max_open",
            default_connections_max_open(),
        )?
        .set_default("log.app.level", default_log_level())?
        .set_default(
            "background.watchdog.period",
            humantime::format_duration(default_watchdog_period()).to_string(),
        )?
        .set_default("background.watchdog.limit", default_watchdog_limit())?
        .set_default(
            "background.watchdog.lock_timeout",
            humantime::format_duration(default_watchdog_lock_timeout()).to_string(),
        )?
        .add_source(File::from(cli.conf.clone()).required(false))
        .add_source(
            Environment::with_prefix("CONF")
                .separator("__")
                .try_parsing(true),
        )
        .set_override("mode.debug", cli.debug)?;

    let settings = builder.build()?;
    settings.try_deserialize().map_err(Into::into)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = load_config(&cli)?;
    let output = serde_json::to_string_pretty(&config)?;
    println!("{}", output);

    Ok(())
}
