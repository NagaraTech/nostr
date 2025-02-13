// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

#[cfg(not(target_arch = "wasm32"))]
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::{AtomicRelayServiceFlags, RelayServiceFlags};
use crate::client::options::DEFAULT_SEND_TIMEOUT;

pub const DEFAULT_RETRY_SEC: u64 = 10;
pub const MIN_RETRY_SEC: u64 = 5;
pub const MAX_ADJ_RETRY_SEC: u64 = 60;
pub const NEGENTROPY_HIGH_WATER_UP: usize = 100;
pub const NEGENTROPY_LOW_WATER_UP: usize = 50;
pub const NEGENTROPY_BATCH_SIZE_DOWN: usize = 50;

/// [`Relay`](super::Relay) options
#[derive(Debug, Clone)]
pub struct RelayOptions {
    /// Proxy
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) proxy: Option<SocketAddr>,
    /// Relay Service Flags
    pub(super) flags: AtomicRelayServiceFlags,
    /// Enable/disable auto reconnection (default: true)
    reconnect: Arc<AtomicBool>,
    /// Retry connection time (default: 10 sec)
    ///
    /// Are allowed values `>=` 5 secs
    retry_sec: Arc<AtomicU64>,
    /// Automatically adjust retry seconds based on success/attempts (default: true)
    adjust_retry_sec: Arc<AtomicBool>,
}

impl Default for RelayOptions {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            proxy: None,
            flags: AtomicRelayServiceFlags::new(RelayServiceFlags::READ | RelayServiceFlags::WRITE),
            reconnect: Arc::new(AtomicBool::new(true)),
            retry_sec: Arc::new(AtomicU64::new(DEFAULT_RETRY_SEC)),
            adjust_retry_sec: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl RelayOptions {
    /// New [`RelayOptions`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set proxy
    #[cfg(not(target_arch = "wasm32"))]
    pub fn proxy(mut self, proxy: Option<SocketAddr>) -> Self {
        self.proxy = proxy;
        self
    }

    /// Set Relay Service Flags
    pub fn flags(mut self, flags: RelayServiceFlags) -> Self {
        self.flags = AtomicRelayServiceFlags::new(flags);
        self
    }

    /// Set read option
    #[deprecated(since = "0.28.0", note = "use `flags` instead")]
    pub fn read(self, read: bool) -> Self {
        if read {
            self.flags.add(RelayServiceFlags::READ);
        } else {
            self.flags.remove(RelayServiceFlags::READ);
        }
        self
    }

    /// Set write option
    #[deprecated(since = "0.28.0", note = "use `flags` instead")]
    pub fn write(self, write: bool) -> Self {
        if write {
            self.flags.add(RelayServiceFlags::WRITE);
        } else {
            self.flags.remove(RelayServiceFlags::WRITE);
        }
        self
    }

    /// Set reconnect option
    pub fn reconnect(self, reconnect: bool) -> Self {
        Self {
            reconnect: Arc::new(AtomicBool::new(reconnect)),
            ..self
        }
    }

    pub(crate) fn get_reconnect(&self) -> bool {
        self.reconnect.load(Ordering::SeqCst)
    }

    /// Set `reconnect` option
    pub fn update_reconnect(&self, reconnect: bool) {
        let _ = self
            .reconnect
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(reconnect));
    }

    /// Set retry seconds option
    pub fn retry_sec(self, retry_sec: u64) -> Self {
        let retry_sec = if retry_sec >= MIN_RETRY_SEC {
            retry_sec
        } else {
            DEFAULT_RETRY_SEC
        };
        Self {
            retry_sec: Arc::new(AtomicU64::new(retry_sec)),
            ..self
        }
    }

    pub(crate) fn get_retry_sec(&self) -> u64 {
        self.retry_sec.load(Ordering::SeqCst)
    }

    /// Set retry_sec option
    pub fn update_retry_sec(&self, retry_sec: u64) {
        if retry_sec >= MIN_RETRY_SEC {
            let _ = self
                .retry_sec
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(retry_sec));
        } else {
            tracing::warn!("Relay options: retry_sec it's less then the minimum value allowed (min: {MIN_RETRY_SEC} secs)");
        }
    }

    /// Automatically adjust retry seconds based on success/attempts (default: true)
    pub fn adjust_retry_sec(self, adjust_retry_sec: bool) -> Self {
        Self {
            adjust_retry_sec: Arc::new(AtomicBool::new(adjust_retry_sec)),
            ..self
        }
    }

    pub(crate) fn get_adjust_retry_sec(&self) -> bool {
        self.adjust_retry_sec.load(Ordering::SeqCst)
    }

    /// Set adjust_retry_sec option
    pub fn update_adjust_retry_sec(&self, adjust_retry_sec: bool) {
        let _ = self
            .adjust_retry_sec
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| {
                Some(adjust_retry_sec)
            });
    }
}

/// [`Relay`](super::Relay) send options
#[derive(Debug, Clone, Copy)]
pub struct RelaySendOptions {
    /// Skip wait for disconnected relay (default: true)
    pub skip_disconnected: bool,
    /// Timeout for sending event (default: 10 secs)
    pub timeout: Duration,
}

impl Default for RelaySendOptions {
    fn default() -> Self {
        Self {
            skip_disconnected: true,
            timeout: DEFAULT_SEND_TIMEOUT,
        }
    }
}

impl RelaySendOptions {
    /// New default [`RelaySendOptions`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Skip wait for disconnected relay (default: true)
    pub fn skip_disconnected(self, value: bool) -> Self {
        Self {
            skip_disconnected: value,
            ..self
        }
    }

    /// Timeout for sending event (default: 10 secs)
    ///
    /// If `None`, the default timeout will be used
    pub fn timeout(self, value: Option<Duration>) -> Self {
        Self {
            timeout: value.unwrap_or(DEFAULT_SEND_TIMEOUT),
            ..self
        }
    }
}

/// Filter options
#[derive(Debug, Clone, Copy, Default)]
pub enum FilterOptions {
    /// Exit on EOSE
    #[default]
    ExitOnEOSE,
    /// After EOSE is received, keep listening for N more events that match the filter, then return
    WaitForEventsAfterEOSE(u16),
    /// After EOSE is received, keep listening for matching events for [`Duration`] more time, then return
    WaitDurationAfterEOSE(Duration),
}

/// Relay Pool Options
#[derive(Debug, Clone, Copy)]
pub struct RelayPoolOptions {
    /// Notification channel size (default: 4096)
    pub notification_channel_size: usize,
    /// Task channel size (default: 4096)
    pub task_channel_size: usize,
    /// Shutdown on [RelayPool](super::pool::RelayPool) drop
    pub shutdown_on_drop: bool,
}

impl Default for RelayPoolOptions {
    fn default() -> Self {
        Self {
            notification_channel_size: 4096,
            task_channel_size: 4096,
            shutdown_on_drop: false,
        }
    }
}

impl RelayPoolOptions {
    /// New default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Shutdown on [`RelayPool`](super::pool::RelayPool) drop
    pub fn shutdown_on_drop(self, value: bool) -> Self {
        Self {
            shutdown_on_drop: value,
            ..self
        }
    }
}

/// Negentropy Sync direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegentropyDirection {
    /// Send events to relay
    Up,
    /// Get events from relay
    Down,
    /// Both send and get events from relay (bidirectional sync)
    Both,
}

impl NegentropyDirection {
    pub(super) fn do_up(&self) -> bool {
        matches!(self, Self::Up | Self::Both)
    }

    pub(super) fn do_down(&self) -> bool {
        matches!(self, Self::Down | Self::Both)
    }
}

/// Negentropy reconciliation options
#[derive(Debug, Clone, Copy)]
pub struct NegentropyOptions {
    pub(super) initial_timeout: Duration,
    pub(super) direction: NegentropyDirection,
}

impl Default for NegentropyOptions {
    fn default() -> Self {
        Self {
            initial_timeout: Duration::from_secs(10),
            direction: NegentropyDirection::Down,
        }
    }
}

impl NegentropyOptions {
    /// New default [`NegentropyOptions`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Timeout to check if negentropy it's supported (default: 10 secs)
    pub fn initial_timeout(mut self, initial_timeout: Duration) -> Self {
        self.initial_timeout = initial_timeout;
        self
    }

    /// Negentropy Sync direction (default: down)
    ///
    /// If `true`, perform the set reconciliation on each side.
    pub fn direction(mut self, direction: NegentropyDirection) -> Self {
        self.direction = direction;
        self
    }
}
