// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

//! NIP01
//!
//! <https://github.com/nostr-protocol/nips/blob/master/01.md>

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;

use bitcoin::secp256k1::{self, XOnlyPublicKey};

use crate::event::id;
use crate::{Filter, Kind, Tag, UncheckedUrl};

/// Raw Event error
#[derive(Debug)]
pub enum Error {
    /// Secp256k1 error
    Secp256k1(secp256k1::Error),
    /// Event ID error
    EventId(id::Error),
    /// Parse Int error
    ParseInt(ParseIntError),
    /// Invalid coordinate
    InvalidCoordinate,
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secp256k1(e) => write!(f, "Secp256k1: {e}"),
            Self::EventId(e) => write!(f, "Event ID: {e}"),
            Self::ParseInt(e) => write!(f, "Parse Int: {e}"),
            Self::InvalidCoordinate => write!(f, "Invalid coordinate"),
        }
    }
}

impl From<secp256k1::Error> for Error {
    fn from(e: secp256k1::Error) -> Self {
        Self::Secp256k1(e)
    }
}

impl From<id::Error> for Error {
    fn from(e: id::Error) -> Self {
        Self::EventId(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

/// Coordinate for event (`a` tag)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Coordinate {
    /// Kind
    pub kind: Kind,
    /// Public Key
    pub pubkey: XOnlyPublicKey,
    /// `d` tag identifier
    ///
    /// Needed for a parametrized replaceable event.
    /// Leave empty for a replaceable event.
    pub identifier: String,
    /// Relays
    pub relays: Vec<String>,
}

impl Coordinate {
    /// Create new event coordinate
    pub fn new(kind: Kind, pubkey: XOnlyPublicKey) -> Self {
        Self {
            kind,
            pubkey,
            identifier: String::new(),
            relays: Vec::new(),
        }
    }

    /// Set a `d` tag identifier
    ///
    /// Needed for a parametrized replaceable event.
    pub fn identifier<S>(mut self, identifier: S) -> Self
    where
        S: Into<String>,
    {
        self.identifier = identifier.into();
        self
    }
}

impl From<Coordinate> for Tag {
    fn from(value: Coordinate) -> Self {
        Self::A {
            kind: value.kind,
            public_key: value.pubkey,
            identifier: value.identifier,
            relay_url: value.relays.first().map(UncheckedUrl::from),
        }
    }
}

impl From<Coordinate> for Filter {
    fn from(value: Coordinate) -> Self {
        if value.identifier.is_empty() {
            Filter::new().kind(value.kind).author(value.pubkey)
        } else {
            Filter::new()
                .kind(value.kind)
                .author(value.pubkey)
                .identifier(value.identifier)
        }
    }
}

impl FromStr for Coordinate {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut kpi = s.split(':');
        if let (Some(kind_str), Some(pubkey_str), Some(identifier)) =
            (kpi.next(), kpi.next(), kpi.next())
        {
            Ok(Self {
                kind: Kind::from_str(kind_str)?,
                pubkey: XOnlyPublicKey::from_str(pubkey_str)?,
                identifier: identifier.to_owned(),
                relays: Vec::new(),
            })
        } else {
            Err(Error::InvalidCoordinate)
        }
    }
}
