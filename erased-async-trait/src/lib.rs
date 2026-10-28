//! A worked example of the **erased-trait pattern** for async traits.
//!
//! Users implement [`Subscriber`] with plain `async fn`. The trait returns
//! `impl Future` (return-position `impl Trait` in traits, RPITIT), so it is
//! not dyn-compatible. The [`Broker`] still stores subscribers of different
//! types side by side, through a crate-private, dyn-compatible twin trait,
//! `ErasedSubscriber`, that boxes each callback's future once:
//!
//! ```text
//! Broker::register::<S>(topic, subscriber)
//!   └─ Arc::new(subscriber)                  Arc<S>
//!        └─ coerced to Arc<dyn ErasedSubscriber>   (S's vtable built here)
//!
//! Broker::subscribe(&sub)
//!   └─ subscriber.subscribe(&sub)            dynamic call through the vtable
//!        └─ S::on_subscribe(&sub)            static call: S's own future
//!        └─ Box::pin(..)                     one allocation per callback
//!   └─ tokio::time::timeout(.., fut).await   cancelled (dropped) on timeout
//! ```
//!
//! ```
//! use std::{sync::{Arc, Mutex}, time::Duration};
//! use erased_async_trait::{Broker, Rejection, Subscriber, Subscription};
//!
//! #[derive(Default)]
//! struct Roster(Mutex<Vec<String>>);
//!
//! impl Subscriber for Roster {
//!     async fn on_subscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
//!         let topic = sub.topic();          // borrowed from `sub`...
//!         tokio::task::yield_now().await;   // ...and held across an await
//!         self.0.lock().unwrap().push(format!("{topic}/{}", sub.id()));
//!         Ok(())
//!     }
//! }
//!
//! # tokio::runtime::Runtime::new().unwrap().block_on(async {
//! let roster = Arc::new(Roster::default());
//! let broker = Broker::new(Duration::from_secs(5));
//! broker.register("orders", roster.clone());
//! broker.subscribe(&Subscription::new("s1", "orders")).await.unwrap();
//! assert_eq!(*roster.0.lock().unwrap(), ["orders/s1"]);
//! # });
//! ```
//!
//! Callback futures must be `Send`, and the compiler checks it at the impl.
//! Holding a `std::sync::MutexGuard` across an `.await` does not compile:
//!
//! ```compile_fail
//! use std::sync::Mutex;
//! use erased_async_trait::{Rejection, Subscriber, Subscription};
//!
//! struct Counter(Mutex<u64>);
//!
//! impl Subscriber for Counter {
//!     async fn on_subscribe(&self, _sub: &Subscription) -> Result<(), Rejection> {
//!         let mut count = self.0.lock().unwrap();
//!         tokio::task::yield_now().await; // the guard lives across this await
//!         *count += 1;
//!         Ok(())
//!     }
//! }
//! ```
//!
//! Releasing the guard before the `.await` fixes it:
//!
//! ```
//! use std::sync::Mutex;
//! use erased_async_trait::{Rejection, Subscriber, Subscription};
//!
//! struct Counter(Mutex<u64>);
//!
//! impl Subscriber for Counter {
//!     async fn on_subscribe(&self, _sub: &Subscription) -> Result<(), Rejection> {
//!         *self.0.lock().unwrap() += 1; // the guard is dropped at the `;`
//!         tokio::task::yield_now().await;
//!         Ok(())
//!     }
//! }
//! ```

mod broker;
mod erased;
mod subscriber;

pub use broker::{Broker, NO_SUBSCRIBER, TIMEOUT};
pub use subscriber::{Rejection, Subscriber, Subscription};
