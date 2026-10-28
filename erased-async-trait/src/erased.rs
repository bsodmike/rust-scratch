//! The crate-private, dyn-compatible twin of [`Subscriber`].
//!
//! [`Subscriber`]'s methods return `impl Future`, a different type for every
//! implementor, so they cannot sit in a vtable. [`ErasedSubscriber`]'s
//! methods return one fixed type, a boxed future, so `dyn ErasedSubscriber`
//! exists and the broker can store subscribers of different types side by
//! side. The blanket impl below gives every [`Subscriber`] this trait for
//! free; users never see it.

use std::pin::Pin;

use crate::subscriber::{Rejection, Subscriber, Subscription};

/// A heap-allocated, type-erased future borrowing for `'a`.
pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub(crate) trait ErasedSubscriber: Send + Sync {
    fn subscribe<'a>(&'a self, sub: &'a Subscription) -> BoxFuture<'a, Result<(), Rejection>>;

    fn unsubscribe<'a>(&'a self, sub: &'a Subscription) -> BoxFuture<'a, ()>;
}

/// The bridge: calls the subscriber's own future (static dispatch) and boxes
/// it once.
///
/// Each callback is called inside the boxed future, not before it. An explicit
/// `fn -> impl Future` impl may run code (and panic) before it returns its
/// future, and the broker's `catch_unwind` only covers what happens while the
/// boxed future is polled. `async move { .. }` only creates a future; the call
/// happens on its first poll.
impl<T: Subscriber> ErasedSubscriber for T {
    fn subscribe<'a>(&'a self, sub: &'a Subscription) -> BoxFuture<'a, Result<(), Rejection>> {
        Box::pin(async move { self.on_subscribe(sub).await })
    }

    fn unsubscribe<'a>(&'a self, sub: &'a Subscription) -> BoxFuture<'a, ()> {
        Box::pin(async move { self.on_unsubscribe(sub).await })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        task::{Context, Poll, Waker},
    };

    use super::*;

    /// Keeps both default callbacks.
    struct Defaults;

    impl Subscriber for Defaults {}

    #[derive(Default)]
    struct Counting {
        subscribed: Mutex<Vec<String>>,
        unsubscribed: Mutex<Vec<String>>,
    }

    impl Subscriber for Counting {
        async fn on_subscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
            tokio::task::yield_now().await;
            self.subscribed.lock().unwrap().push(sub.id().to_string());
            Ok(())
        }

        async fn on_unsubscribe(&self, sub: &Subscription) {
            tokio::task::yield_now().await;
            self.unsubscribed.lock().unwrap().push(sub.id().to_string());
        }
    }

    /// An explicit `fn -> impl Future` impl: records that it was called
    /// before returning its future.
    #[derive(Default)]
    struct Eager {
        called: AtomicBool,
    }

    impl Subscriber for Eager {
        fn on_subscribe<'a>(
            &'a self,
            _sub: &'a Subscription,
        ) -> impl Future<Output = Result<(), Rejection>> + Send + 'a {
            self.called.store(true, Ordering::SeqCst);
            std::future::ready(Ok(()))
        }
    }

    #[test]
    fn a_stored_subscriber_is_a_fat_pointer() {
        // Data pointer + vtable pointer.
        assert_eq!(
            size_of::<Arc<dyn ErasedSubscriber>>(),
            2 * size_of::<usize>()
        );
    }

    #[tokio::test]
    async fn the_adapter_runs_the_default_callbacks() {
        let erased: Arc<dyn ErasedSubscriber> = Arc::new(Defaults);
        let sub = Subscription::new("s1", "t");
        assert_eq!(erased.subscribe(&sub).await, Ok(()));
        erased.unsubscribe(&sub).await;
    }

    #[tokio::test]
    async fn the_adapter_runs_overridden_callbacks() {
        let counting = Arc::new(Counting::default());
        // Arc<Arc<Counting>>: the outer Arc is the storage, the inner one the
        // forwarding impl. Both reach the same Counting.
        let erased: Arc<dyn ErasedSubscriber> = Arc::new(counting.clone());
        let sub = Subscription::new("s1", "t");
        assert_eq!(erased.subscribe(&sub).await, Ok(()));
        erased.unsubscribe(&sub).await;
        assert_eq!(*counting.subscribed.lock().unwrap(), ["s1"]);
        assert_eq!(*counting.unsubscribed.lock().unwrap(), ["s1"]);
    }

    #[tokio::test]
    async fn subscribers_of_different_types_share_one_collection() {
        let counting = Arc::new(Counting::default());
        let all: Vec<Arc<dyn ErasedSubscriber>> =
            vec![Arc::new(Defaults), Arc::new(counting.clone())];
        let sub = Subscription::new("s1", "t");
        for subscriber in &all {
            assert_eq!(subscriber.subscribe(&sub).await, Ok(()));
        }
        assert_eq!(*counting.subscribed.lock().unwrap(), ["s1"]);
    }

    #[tokio::test]
    async fn nothing_runs_until_the_boxed_future_is_polled() {
        let counting = Arc::new(Counting::default());
        let erased: Arc<dyn ErasedSubscriber> = Arc::new(counting.clone());
        let sub = Subscription::new("s1", "t");
        let pending = erased.subscribe(&sub);
        assert!(counting.subscribed.lock().unwrap().is_empty());
        drop(pending);
        assert!(counting.subscribed.lock().unwrap().is_empty());
    }

    #[test]
    fn the_callback_is_called_on_the_first_poll_not_before() {
        let eager = Arc::new(Eager::default());
        let erased: Arc<dyn ErasedSubscriber> = Arc::new(eager.clone());
        let sub = Subscription::new("s1", "t");

        let mut pending = erased.subscribe(&sub);
        assert!(
            !eager.called.load(Ordering::SeqCst),
            "building the boxed future must not call on_subscribe"
        );

        let poll = pending
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()));
        assert_eq!(poll, Poll::Ready(Ok(())));
        assert!(eager.called.load(Ordering::SeqCst));
    }
}
