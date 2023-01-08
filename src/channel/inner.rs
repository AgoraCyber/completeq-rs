use std::{
    collections::HashMap,
    task::{Poll, Waker},
};

use crate::{
    result::{EmitInnerResult, ReceiveResult},
    user_event::UserEvent,
};

#[derive(Default)]
struct Channel<Argument> {
    /// Channel bound receiver counter.
    ///
    /// if this value decrease to `0`, this framework will automatic release current channel instance.
    ref_count: usize,
    /// The max length of channel message waiting queue.
    max_len: usize,
    /// Channel receiver wakers
    ///
    /// The [`HashMap`] `key` is receiver id.
    receivers: HashMap<usize, Waker>,
    /// Queue of messages waiting to be read by the receiver
    pending_msgs: Vec<Argument>,
    /// Pending senders
    senders: Vec<Waker>,
}

/// CompleteQ inner implementation.
#[derive(Default)]
pub(crate) struct CompleteQImpl<E: UserEvent> {
    channels: HashMap<E::ID, Channel<E::Argument>>,
}

impl<E: UserEvent> CompleteQImpl<E> {
    /// Send one completed event
    ///
    /// If pending msgs >= max_len , will return [`Pending`](super::emit::EmitInnerResult::Pending)
    pub fn complete_one(
        &mut self,
        event_id: E::ID,
        event_arg: E::Argument,
        waker: Waker,
    ) -> EmitInnerResult<E> {
        log::trace!("complete_one event_id({})", event_id);
        if let Some(channel) = self.channels.get_mut(&event_id) {
            if channel.pending_msgs.len() >= channel.max_len {
                log::trace!("complete_one event_id({}) -- pending", event_id);
                channel.senders.push(waker);
                return EmitInnerResult::Pending(event_arg);
            }

            log::trace!("complete_one event_id({}) -- ready", event_id);
            channel.pending_msgs.push(event_arg);

            if let Some(key) = channel.receivers.keys().next().map(|k| *k) {
                channel.receivers.remove(&key).unwrap().wake_by_ref();
            }

            return EmitInnerResult::Completed;
        }

        log::trace!("complete_one event_id({}) -- closed", event_id);

        EmitInnerResult::Closed
    }

    /// Try poll one completed event.
    ///
    /// If the channel's waiting queue length is zero,
    /// this method will return [`Pending`](Poll::Pending) and cache [`waker`](Waker) parameter.
    pub fn poll_once(
        &mut self,
        receiver_id: usize,
        event_id: E::ID,
        waker: Waker,
    ) -> Poll<ReceiveResult<E>> {
        log::trace!(
            "poll_one receiver_id({}) event_id({})",
            receiver_id,
            event_id
        );

        let channel = self
            .channels
            .get_mut(&event_id)
            .expect("Call open_channel first");

        if !channel.pending_msgs.is_empty() {
            let argument = channel.pending_msgs.swap_remove(0);

            // wakeup one pending sender
            if !channel.senders.is_empty() {
                channel.senders.swap_remove(0).wake_by_ref();
            }

            return Poll::Ready(ReceiveResult::Success(argument));
        } else {
            channel.receivers.insert(receiver_id, waker);
            return Poll::Pending;
        }
    }

    /// Open event_id binding channel, and return receiver `Counter`
    ///
    /// Warning !!!  same receiver `MUST NOT` call this method twice.
    ///
    /// # Arguments
    ///
    /// * max_len - Reset waiting quene length .
    ///             So the final length of this bound channel is determined by the last opened receiver
    ///
    pub fn open_channel(&mut self, event_id: E::ID, max_len: usize) -> usize {
        self.channels
            .entry(event_id)
            .and_modify(|c| {
                c.ref_count += 1;

                c.max_len = max_len;
            })
            .or_insert(Channel {
                ref_count: 1,
                max_len,
                ..Default::default()
            })
            .ref_count
    }

    /// Reduce channel ref_count, if necessary remove channel from memory.
    pub fn close_channel(&mut self, event_id: E::ID) -> usize {
        if let Some(channel) = self.channels.get_mut(&event_id) {
            channel.max_len -= 1;

            if channel.max_len == 0 {
                self.channels.remove(&event_id);

                return 0;
            } else {
                return channel.max_len;
            }
        }

        0
    }

    /// According to parameter `receiver_id`, delete the pending poll of the corresponding receiver
    pub fn remove_pending_poll(&mut self, receiver_id: usize, event_id: E::ID) {
        if let Some(channel) = self.channels.get_mut(&event_id) {
            if let Some(waker) = channel.receivers.remove(&receiver_id) {
                waker.wake_by_ref();
            }
        }
    }
}
