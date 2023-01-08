use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompleteQError {
    #[error("Event channel closed")]
    PipeBroken,
    #[error("Read from event channel timed out")]
    Timeout,
    #[error("Open oneshot channel twice")]
    OpenChannelTwice,
}
