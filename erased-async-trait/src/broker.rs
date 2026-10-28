//! The runtime side: stores subscribers of any type and runs their callbacks.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    erased::ErasedSubscriber,
    subscriber::{Rejection, Subscriber, Subscription},
};

/// The code of a subscription to a topic no subscriber is registered for.
pub const NO_SUBSCRIBER: &str = "NO_SUBSCRIBER";

/// The code of a callback that did not finish within the broker's timeout.
pub const TIMEOUT: &str = "TIMEOUT";

/// Routes subscriptions to the subscriber registered for their topic.
///
/// Each topic can have a subscriber of a different type. They are all stored
/// as `Arc<dyn ErasedSubscriber>`, a crate-private trait object.
pub struct Broker {
    subscribers: Mutex<HashMap<String, Arc<dyn ErasedSubscriber>>>,
    timeout: Duration,
}

impl Broker {
    /// A broker whose callbacks are cancelled after `timeout`.
    pub fn new(timeout: Duration) -> Self {
        Self {
            subscribers: Mutex::new(HashMap::new()),
            timeout,
        }
    }

    /// Registers `subscriber` for `topic`, replacing any previous one.
    ///
    /// Pass an `Arc` of a subscriber to share it between topics.
    pub fn register<S: Subscriber>(&self, topic: impl Into<String>, subscriber: S) {
        // Unsizing coercion: Arc<S> becomes Arc<dyn ErasedSubscriber>, and the
        // compiler builds S's vtable for ErasedSubscriber here. From now on
        // S's type is erased.
        let subscriber: Arc<dyn ErasedSubscriber> = Arc::new(subscriber);
        self.subscribers
            .lock()
            .unwrap()
            .insert(topic.into(), subscriber);
    }

    /// Runs the topic's `on_subscribe`. A rejection, an unknown topic
    /// ([`NO_SUBSCRIBER`]) or a callback past the timeout ([`TIMEOUT`]) is an
    /// error. A callback past the timeout is cancelled: its future is dropped.
    pub async fn subscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
        let subscriber = self.subscriber(sub.topic())?;
        match tokio::time::timeout(self.timeout, subscriber.subscribe(sub)).await {
            Ok(result) => result,
            Err(_) => Err(self.timed_out("on_subscribe", sub)),
        }
    }

    /// Runs the topic's `on_unsubscribe`. Only an unknown topic or a callback
    /// past the timeout is an error.
    pub async fn unsubscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
        let subscriber = self.subscriber(sub.topic())?;
        tokio::time::timeout(self.timeout, subscriber.unsubscribe(sub))
            .await
            .map_err(|_| self.timed_out("on_unsubscribe", sub))
    }

    fn subscriber(&self, topic: &str) -> Result<Arc<dyn ErasedSubscriber>, Rejection> {
        // Clone the Arc out and release the lock here, before any `.await`:
        // a std MutexGuard is not Send, and holding it across an await would
        // make `subscribe`'s future !Send (and block other callers).
        self.subscribers
            .lock()
            .unwrap()
            .get(topic)
            .cloned()
            .ok_or_else(|| {
                Rejection::new(NO_SUBSCRIBER, format!("no subscriber for topic {topic}"))
            })
    }

    fn timed_out(&self, callback: &str, sub: &Subscription) -> Rejection {
        Rejection::new(
            TIMEOUT,
            format!(
                "{callback} for {} did not finish within {:?}",
                sub.id(),
                self.timeout
            ),
        )
    }
}
