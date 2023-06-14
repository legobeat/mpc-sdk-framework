//! Server configuration.
use mpc_relay_protocol::{decode_keypair, hex, snow::Keypair};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::{Error, Result};

/// Configuration for the web server.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Path to the server key.
    pub key: PathBuf,

    /// Settings for session management.
    pub session: SessionConfig,

    /// Configuration for TLS encryption.
    pub tls: Option<TlsConfig>,

    /// Allow access to clients with these
    /// public keys.
    pub allow: Option<Vec<AccessKey>>,

    /// Deny access to clients with these
    /// public keys.
    pub deny: Option<Vec<AccessKey>>,
}

impl ServerConfig {
    /// Determine if a public key is allowed access.
    pub fn is_allowed_access(&self, key: impl AsRef<[u8]>) -> bool {
        //let restricted = self.allow.is_some() || self.deny.is_some();

        if let Some(deny) = &self.deny {
            if deny.iter().any(|k| k.public_key == key.as_ref()) {
                return false;
            }
        }

        if let Some(allow) = &self.allow {
            if allow.iter().any(|k| k.public_key == key.as_ref()) {
                return true;
            }
            false
        } else {
            true
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct AccessKey {
    #[serde(with = "hex::serde")]
    public_key: Vec<u8>,
}

/// Certificate and key for TLS.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to the certificate.
    pub cert: PathBuf,
    /// Path to the certificate key file.
    pub key: PathBuf,
}

/// Configuration for server sessions.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Timeout for sessions in seconds.
    ///
    /// Sessions that have not seen any message activity
    /// for this amount of time are marked for deletion.
    ///
    /// Default is 5 minutes.
    pub timeout: u64,

    /// Interval in seconds to reap expired sessions.
    ///
    /// Default is every 15 minutes.
    pub reap_interval: u64,

    /// The interval used to poll a session for the ready
    /// and active states.
    ///
    /// A session is ready when all participants have completed
    /// the server handshake and is active when all participants
    /// have established their peer connections.
    pub wait_interval: u64,

    /// Wait timeout controls the timeout when waiting
    /// for all clients in a session to become active.
    ///
    /// Default is 5 minutes.
    pub wait_timeout: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: 300,
            reap_interval: 900,
            wait_interval: 15,
            wait_timeout: 300,
        }
    }
}

impl ServerConfig {
    /// Load a server config from a file path.
    pub async fn load<P: AsRef<Path>>(
        path: P,
    ) -> Result<(Self, Keypair)> {
        if !fs::try_exists(path.as_ref()).await? {
            return Err(Error::NotFile(path.as_ref().to_path_buf()));
        }

        let contents = fs::read_to_string(path.as_ref()).await?;
        let mut config: ServerConfig = toml::from_str(&contents)?;

        if config.key == PathBuf::default() {
            return Err(Error::KeyFileRequired);
        }

        let dir = Self::directory(path.as_ref())?;

        if config.key.is_relative() {
            config.key = dir.join(&config.key).canonicalize()?;
        }

        if !fs::try_exists(&config.key).await? {
            return Err(Error::KeyNotFound(config.key.clone()));
        }

        let contents = fs::read_to_string(&config.key).await?;
        let keypair = decode_keypair(contents)?;

        if let Some(tls) = config.tls.as_mut() {
            if tls.cert.is_relative() {
                tls.cert = dir.join(&tls.cert).canonicalize()?;
            }
            if tls.key.is_relative() {
                tls.key = dir.join(&tls.key).canonicalize()?;
            }
        }

        Ok((config, keypair))
    }

    /// Parent directory of the configuration file.
    fn directory(file: impl AsRef<Path>) -> Result<PathBuf> {
        file.as_ref()
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| Error::NoParentDir)
    }
}
