//! The pattern's guarantees, through the public API only. The adapter's own
//! tests are unit tests in `src/erased.rs` (the trait is crate-private).

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use erased_async_trait::{Broker, NO_SUBSCRIBER, Rejection, Subscriber, Subscription, TIMEOUT};

const WAIT: Duration = Duration::from_secs(30);

/// Holds a borrow of `sub` across awaits in both callbacks, with no clone
/// first. Rejects a subscription with a `reject` param.
#[derive(Clone, Default)]
struct Recorder {
    added: Arc<Mutex<Vec<String>>>,
    removed: Arc<Mutex<Vec<String>>>,
}

impl Subscriber for Recorder {
    async fn on_subscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
        let topic: &str = sub.topic(); // a borrow of `sub`...
        tokio::task::yield_now().await; // ...held across an await
        tokio::time::sleep(Duration::from_millis(5)).await; // ...and another
        if let Some(reason) = sub.param("reject") {
            return Err(Rejection::new("REJECTED", reason));
        }
        self.added
            .lock()
            .unwrap()
            .push(format!("{topic}/{}", sub.id()));
        Ok(())
    }

    async fn on_unsubscribe(&self, sub: &Subscription) {
        let id: &str = sub.id();
        tokio::task::yield_now().await;
        self.removed.lock().unwrap().push(id.to_string());
    }
}

// ─── borrowing across .await ───────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn a_subscriber_borrows_its_subscription_across_await() {
    let recorder = Recorder::default();
    let broker = Broker::new(WAIT);
    broker.register("orders", recorder.clone());
    let sub = Subscription::new("s1", "orders");

    broker.subscribe(&sub).await.unwrap();
    broker.unsubscribe(&sub).await.unwrap();

    assert_eq!(*recorder.added.lock().unwrap(), ["orders/s1"]);
    assert_eq!(*recorder.removed.lock().unwrap(), ["s1"]);
}

/// Records the address of the `Subscription` it is handed.
#[derive(Default)]
struct Identity {
    seen: Mutex<Option<usize>>,
}

impl Subscriber for Identity {
    async fn on_subscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
        tokio::task::yield_now().await;
        *self.seen.lock().unwrap() = Some(std::ptr::from_ref(sub) as usize);
        Ok(())
    }
}

/// Through the erased adapter and `Arc` forwarding, the subscriber gets the
/// caller's own `&Subscription`: a borrow, never a clone.
#[tokio::test]
async fn the_callers_reference_reaches_the_subscriber_unchanged() {
    let identity = Arc::new(Identity::default());
    let broker = Broker::new(WAIT);
    broker.register("t", identity.clone());
    let sub = Subscription::new("s1", "t");

    broker.subscribe(&sub).await.unwrap();

    assert_eq!(
        *identity.seen.lock().unwrap(),
        Some(std::ptr::from_ref(&sub) as usize)
    );
}

// ─── Send futures ──────────────────────────────────────────────────────────

fn assert_send<T: Send>(_: &T) {}

#[tokio::test]
async fn every_future_on_the_path_is_send() {
    let recorder = Recorder::default();
    let shared = Arc::new(Recorder::default());
    let sub = Subscription::new("s1", "t");
    let broker = Arc::new(Broker::new(WAIT));

    // Compile-time: these lines only build if each future is Send.
    assert_send(&recorder.on_subscribe(&sub));
    assert_send(&recorder.on_unsubscribe(&sub));
    assert_send(&<Arc<Recorder> as Subscriber>::on_subscribe(&shared, &sub));
    assert_send(&<Arc<Recorder> as Subscriber>::on_unsubscribe(
        &shared, &sub,
    ));
    assert_send(&broker.subscribe(&sub));
    assert_send(&broker.unsubscribe(&sub));

    // Runtime: tokio::spawn requires a Send + 'static future.
    broker.register("t", recorder.clone());
    let task = tokio::spawn({
        let broker = broker.clone();
        let sub = sub.clone();
        async move { broker.subscribe(&sub).await }
    });
    assert_eq!(task.await.unwrap(), Ok(()));
    assert_eq!(*recorder.added.lock().unwrap(), ["t/s1"]);
}

// ─── forwarding through Arc ────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn one_arc_serves_several_topics() {
    let shared = Arc::new(Recorder::default());
    let broker = Broker::new(WAIT);
    broker.register("orders", shared.clone());
    broker.register("payments", shared.clone());
    assert_eq!(Arc::strong_count(&shared), 3);

    broker
        .subscribe(&Subscription::new("s1", "orders"))
        .await
        .unwrap();
    broker
        .subscribe(&Subscription::new("s2", "payments"))
        .await
        .unwrap();

    assert_eq!(*shared.added.lock().unwrap(), ["orders/s1", "payments/s2"]);
}

#[tokio::test(start_paused = true)]
async fn calling_through_an_arc_reaches_the_inner_subscriber() {
    let shared = Arc::new(Recorder::default());
    let sub = Subscription::new("s1", "t");

    // Static dispatch, no broker: the Arc impl returns the inner future as is.
    <Arc<Recorder> as Subscriber>::on_subscribe(&shared, &sub)
        .await
        .unwrap();
    <Arc<Recorder> as Subscriber>::on_unsubscribe(&shared, &sub).await;

    assert_eq!(*shared.added.lock().unwrap(), ["t/s1"]);
    assert_eq!(*shared.removed.lock().unwrap(), ["s1"]);
}

// ─── errors ────────────────────────────────────────────────────────────────

#[tokio::test(start_paused = true)]
async fn a_rejection_reaches_the_caller_unchanged() {
    let recorder = Recorder::default();
    let broker = Broker::new(WAIT);
    broker.register("t", recorder.clone());

    let rejection = broker
        .subscribe(&Subscription::new("s1", "t").with_param("reject", "quota exceeded"))
        .await
        .unwrap_err();

    assert_eq!(rejection, Rejection::new("REJECTED", "quota exceeded"));
    assert_eq!(rejection.to_string(), "REJECTED: quota exceeded");
    assert!(recorder.added.lock().unwrap().is_empty());
}

#[tokio::test]
async fn an_unknown_topic_is_no_subscriber() {
    let broker = Broker::new(WAIT);
    let sub = Subscription::new("s1", "nowhere");

    assert_eq!(
        broker.subscribe(&sub).await.unwrap_err().code,
        NO_SUBSCRIBER
    );
    assert_eq!(
        broker.unsubscribe(&sub).await.unwrap_err().code,
        NO_SUBSCRIBER
    );
}

// ─── cancellation ──────────────────────────────────────────────────────────

/// Sets its flag when dropped: proves a future was cancelled.
struct SetOnDrop(Arc<AtomicBool>);

impl Drop for SetOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

/// Never finishes a subscription with a `stall` param.
#[derive(Clone, Default)]
struct Stalls {
    entered: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,
    done: Arc<Mutex<Vec<String>>>,
}

impl Subscriber for Stalls {
    async fn on_subscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
        if sub.param("stall").is_some() {
            let _guard = SetOnDrop(self.dropped.clone());
            self.entered.store(true, Ordering::SeqCst);
            std::future::pending::<()>().await;
        }
        self.done.lock().unwrap().push(sub.id().to_string());
        Ok(())
    }
}

#[tokio::test(start_paused = true)]
async fn a_callback_past_the_timeout_is_dropped_and_the_next_one_runs() {
    let stalls = Stalls::default();
    let broker = Broker::new(WAIT);
    broker.register("t", stalls.clone());

    let rejection = broker
        .subscribe(&Subscription::new("s1", "t").with_param("stall", ""))
        .await
        .unwrap_err();

    assert_eq!(rejection.code, TIMEOUT);
    assert!(
        stalls.dropped.load(Ordering::SeqCst),
        "the stalled future was dropped"
    );
    broker
        .subscribe(&Subscription::new("s2", "t"))
        .await
        .unwrap();
    assert_eq!(*stalls.done.lock().unwrap(), ["s2"]);
}

#[tokio::test]
async fn aborting_the_caller_cancels_the_callback() {
    let stalls = Stalls::default();
    let broker = Arc::new(Broker::new(WAIT));
    broker.register("t", stalls.clone());

    let task = tokio::spawn({
        let broker = broker.clone();
        async move {
            let sub = Subscription::new("s1", "t").with_param("stall", "");
            broker.subscribe(&sub).await
        }
    });
    while !stalls.entered.load(Ordering::SeqCst) {
        tokio::task::yield_now().await;
    }
    assert!(!stalls.dropped.load(Ordering::SeqCst));

    task.abort();

    assert!(task.await.unwrap_err().is_cancelled());
    assert!(
        stalls.dropped.load(Ordering::SeqCst),
        "the aborted future was dropped"
    );
}

#[tokio::test]
async fn losing_a_select_race_cancels_the_callback() {
    let stalls = Stalls::default();
    let broker = Broker::new(WAIT);
    broker.register("t", stalls.clone());
    let sub = Subscription::new("s1", "t").with_param("stall", "");

    tokio::select! {
        _ = broker.subscribe(&sub) => unreachable!("the callback never finishes"),
        () = async {
            while !stalls.entered.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        } => {}
    }

    assert!(
        stalls.dropped.load(Ordering::SeqCst),
        "the losing future was dropped"
    );
}
