use std::time::Duration;

pub trait Timer {
    /// Create a timer when `duration` time interval has elapsed. trigger the `callback`
    fn interval(duration: Duration, callback: impl FnOnce() + Send + Sync + 'static);
}
