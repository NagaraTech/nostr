// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use tracing::Level;
use uniffi::Enum;

use crate::error::Result;

#[derive(Enum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => Self::TRACE,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
        }
    }
}

#[uniffi::export]
pub fn init_logger(level: LogLevel) -> Result<()> {
    let level: Level = level.into();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
