//! Drive multi-party computation protocols to completion.
#![deny(missing_docs)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
use async_trait::async_trait;
use mpc_client::Event;

mod bridge;
mod error;
mod round;
mod session;

pub use bridge::{wait_for_driver, wait_for_close, wait_for_session_finish};
pub(crate) use bridge::Bridge;
pub use error::Error;
pub(crate) use round::{Round, RoundBuffer, RoundMsg};
pub use session::{
    wait_for_session, SessionEventHandler, SessionHandler,
    SessionInitiator, SessionParticipant,
};

/// Result type for the driver library.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "gg20")]
pub mod gg20;

#[cfg(feature = "gg20")]
#[doc(hidden)]
pub use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020;

#[cfg(feature = "gg20")]
#[doc(hidden)]
pub use curv;

/// Drives a protocol to completion bridging between
/// the network transport and local computation.
#[async_trait]
pub trait Driver {
    /// Error type.
    type Error: std::fmt::Debug + From<mpc_client::Error>;

    /// Output yielded when the driver completes.
    type Output;

    /// Handle an incoming event.
    async fn handle_event(
        &mut self,
        event: Event,
    ) -> std::result::Result<Option<Self::Output>, Self::Error>;

    /// Start running the protocol.
    async fn execute(
        &mut self,
    ) -> std::result::Result<(), Self::Error>;
}

/// Trait for implementations that drive
/// protocol to completion.
pub(crate) trait ProtocolDriver {
    /// Error type for results.
    type Error: std::fmt::Debug
        + From<mpc_client::Error>
        + From<Box<crate::Error>>;
    /// Incoming message type.
    type Incoming: From<Self::Outgoing>;
    /// Outgoing message type.
    type Outgoing: std::fmt::Debug + round::Round;
    /// Output when the protocol is completed.
    type Output;

    /// Handle an incoming message.
    fn handle_incoming(
        &mut self,
        message: Self::Incoming,
    ) -> std::result::Result<(), Self::Error>;

    /*
    /// Determine if the protocol wants to proceed.
    fn wants_to_proceed(&self) -> bool;
    */

    /// Proceed to the next round.
    fn proceed(
        &mut self,
    ) -> std::result::Result<Vec<Self::Outgoing>, Self::Error>;

    /// Complete the protocol and get the output.
    fn finish(self)
        -> std::result::Result<Self::Output, Self::Error>;
}

#[doc(hidden)]
/// Compute the address of an uncompressed public key (65 bytes).
pub fn address(public_key: &[u8]) -> String {
    use mpc_protocol::hex;
    use sha3::{Digest, Keccak256};
    // Remove the leading 0x04
    let bytes = &public_key[1..];
    let digest = Keccak256::digest(bytes);
    let final_bytes = &digest[12..];
    format!("0x{}", hex::encode(final_bytes))
}

