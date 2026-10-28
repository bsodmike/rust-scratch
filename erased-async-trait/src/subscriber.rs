//! The public contract: what users implement and what they receive.

use std::{collections::BTreeMap, fmt, sync::Arc};

/// One subscription to a topic, as a [`Subscriber`] sees it.
///
/// Cheap to clone (an `Arc`), so a subscriber can keep it in its own index.
/// Two subscriptions are equal when their ids are.
#[derive(Clone, Debug)]
pub struct Subscription {
    inner: Arc<Inner>,
}

#[derive(Clone, Debug)]
struct Inner {
    id: String,
    topic: String,
    params: BTreeMap<String, String>,
}

impl Subscription {
    pub fn new(id: impl Into<String>, topic: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Inner {
                id: id.into(),
                topic: topic.into(),
                params: BTreeMap::new(),
            }),
        }
    }

    /// Adds a parameter the subscriber can read with [`Subscription::param`].
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        Arc::make_mut(&mut self.inner)
            .params
            .insert(key.into(), value.into());
        self
    }

    pub fn id(&self) -> &str {
        &self.inner.id
    }

    pub fn topic(&self) -> &str {
        &self.inner.topic
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.inner.params.get(key).map(String::as_str)
    }
}

impl PartialEq for Subscription {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl Eq for Subscription {}

/// A subscriber's refusal of a subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    pub code: String,
    pub message: String,
}

impl Rejection {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Rejection {}

/// Callbacks run when a subscription to a topic is added or removed.
///
/// Implement them with plain `async fn`. Both may borrow `self` and `sub`
/// across `.await`, because the returned future is tied to the lifetime `'a`
/// of both borrows. Their futures must be `Send`: the compiler checks this at
/// your `impl`, so holding a `std::sync::MutexGuard` across an `.await` is
/// rejected there.
///
/// The trait is not dyn-compatible (each impl returns its own future type),
/// so there is no `dyn Subscriber`. Hand the concrete type, or an `Arc` of
/// it, to [`Broker::register`](crate::Broker::register).
pub trait Subscriber: Send + Sync + 'static {
    /// A subscription was requested. Returning an error rejects it.
    fn on_subscribe<'a>(
        &'a self,
        _sub: &'a Subscription,
    ) -> impl Future<Output = Result<(), Rejection>> + Send + 'a {
        async { Ok(()) }
    }

    /// A subscription was removed. Cannot fail.
    fn on_unsubscribe<'a>(
        &'a self,
        _sub: &'a Subscription,
    ) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }
}

/// Share one subscriber between topics. Forwards the call as is: the inner
/// subscriber's future is returned unchanged, with no box and no clone of
/// `sub`.
impl<T: Subscriber> Subscriber for Arc<T> {
    fn on_subscribe<'a>(
        &'a self,
        sub: &'a Subscription,
    ) -> impl Future<Output = Result<(), Rejection>> + Send + 'a {
        (**self).on_subscribe(sub)
    }

    fn on_unsubscribe<'a>(&'a self, sub: &'a Subscription) -> impl Future<Output = ()> + Send + 'a {
        (**self).on_unsubscribe(sub)
    }
}
