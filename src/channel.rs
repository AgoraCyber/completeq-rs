use std::{
    sync::{atomic::AtomicUsize, Arc, Mutex},
    time::Duration,
};

use crate::user_event::{AutoIncEvent, UserEvent};

mod inner;
use async_timer::{Timer, TimerWithContext};
use inner::*;
mod receiver;
pub use receiver::*;
mod sender;
pub use sender::*;

/// CompleteQ structure is a central scheduler for certain types of completion events
///
/// The generic parameter `E` represents a user-defined event type
#[derive(Clone)]
pub struct CompleteQ<E: UserEvent> {
    event: E,
    /// receiver id generator
    receiver_id_seq: Arc<AtomicUsize>,

    inner: Arc<Mutex<CompleteQImpl<E>>>,
}

impl<E: UserEvent> CompleteQ<E>
where
    E: 'static,
{
    pub fn new() -> Self {
        Self {
            event: E::default(),
            receiver_id_seq: Arc::new(AtomicUsize::new(1)),
            inner: Default::default(),
        }
    }

    pub fn cancel_all(&self) {
        self.inner.lock().unwrap().cancel_all();
    }

    pub fn complete_one(&self, event_id: E::ID, event_arg: E::Argument) -> EventSend<E> {
        EventSend::new(event_id, event_arg, self.inner.clone())
    }

    /// Create a new event receiver with provide event_id
    pub fn wait_for(
        &self,
        event_id: E::ID,
        max_len: usize,
    ) -> EventReceiver<E, async_timer::hashed::Timeout> {
        EventReceiver::new(
            event_id,
            max_len,
            self.receiver_id_seq.clone(),
            self.inner.clone(),
            None,
        )
    }

    /// [`wait_for`](CompleteQ::wait_for)  operation with timeout
    pub fn wait_for_timeout<T: TimerWithContext>(
        &self,
        event_id: E::ID,
        max_len: usize,
        timeout: Duration,
    ) -> EventReceiver<E, T> {
        self.wait_for_with_timer(event_id, max_len, T::new(timeout))
    }

    pub fn wait_for_timeout_with_context<T: TimerWithContext, C>(
        &self,
        event_id: E::ID,
        max_len: usize,
        timeout: Duration,
        context: C,
    ) -> EventReceiver<E, T>
    where
        C: AsMut<T::Context>,
    {
        self.wait_for_with_timer(event_id, max_len, T::new_with_context(timeout, context))
    }

    pub fn wait_for_with_timer<T: Timer>(
        &self,
        event_id: E::ID,
        max_len: usize,
        timer: T,
    ) -> EventReceiver<E, T> {
        let receiver = EventReceiver::new(
            event_id,
            max_len,
            self.receiver_id_seq.clone(),
            self.inner.clone(),
            Some(timer),
        );

        receiver
    }
}

impl<E: AutoIncEvent> CompleteQ<E>
where
    E: 'static,
{
    /// Create a new event receiver with automatic generate event_id
    pub fn wait_one(&mut self, max_len: usize) -> EventReceiver<E, async_timer::hashed::Timeout> {
        let event_id = self.event.next();
        self.wait_for(event_id, max_len)
    }

    pub fn wait_one_with_timer<T: Timer>(
        &mut self,
        max_len: usize,
        timer: T,
    ) -> EventReceiver<E, T> {
        let event_id = self.event.next();

        self.wait_for_with_timer(event_id, max_len, timer)
    }

    pub fn wait_one_timeout<T: Timer>(
        &mut self,
        max_len: usize,
        duration: Duration,
    ) -> EventReceiver<E, T> {
        let event_id = self.event.next();

        self.wait_for_with_timer(event_id, max_len, T::new(duration))
    }

    pub fn wait_one_timeout_with_context<T: TimerWithContext, C>(
        &mut self,
        max_len: usize,
        duration: Duration,
        context: C,
    ) -> EventReceiver<E, T>
    where
        C: AsMut<T::Context>,
    {
        let event_id = self.event.next();

        self.wait_for_with_timer(event_id, max_len, T::new_with_context(duration, context))
    }
}

#[cfg(test)]
mod tests {
    use crate::{error::CompleteQError, user_event::RPCResponser};

    use super::CompleteQ;

    #[derive(Default)]
    struct NullArgument;

    type Event = RPCResponser<NullArgument>;

    #[async_std::test]
    async fn one_send_one_recv() -> anyhow::Result<()> {
        _ = pretty_env_logger::try_init();

        let mut q = CompleteQ::<Event>::new();

        let receiver = q.wait_one(10);

        let sender = receiver.sender();

        async_std::task::spawn(async move {
            sender.send(Default::default()).await.completed()?;

            Ok::<(), CompleteQError>(())
        });

        receiver.await.success()?;

        Ok(())
    }
}
