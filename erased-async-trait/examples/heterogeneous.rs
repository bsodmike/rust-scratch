//! Two subscriber types behind one broker, one of them shared across topics.
//!
//! Run with `cargo run --example heterogeneous`.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use erased_async_trait::{Broker, Rejection, Subscriber, Subscription};

/// Indexes subscription ids by topic. One `Arc<Roster>` serves two topics.
#[derive(Default)]
struct Roster {
    by_topic: Mutex<BTreeMap<String, Vec<String>>>,
}

impl Subscriber for Roster {
    async fn on_subscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
        let topic: &str = sub.topic(); // borrowed from `sub`...
        tokio::time::sleep(Duration::from_millis(10)).await; // ...and held across this await
        self.by_topic
            .lock()
            .unwrap()
            .entry(topic.to_string())
            .or_default()
            .push(sub.id().to_string());
        Ok(())
    }

    async fn on_unsubscribe(&self, sub: &Subscription) {
        if let Some(ids) = self.by_topic.lock().unwrap().get_mut(sub.topic()) {
            ids.retain(|id| id != sub.id());
        }
    }
}

/// Requires a `user` param and logs who subscribed. Keeps the default
/// `on_unsubscribe`.
struct Audit;

impl Subscriber for Audit {
    async fn on_subscribe(&self, sub: &Subscription) -> Result<(), Rejection> {
        let Some(user) = sub.param("user") else {
            return Err(Rejection::new(
                "MISSING_USER",
                format!("{} has no user param", sub.id()),
            ));
        };
        tokio::task::yield_now().await;
        println!("  [audit] {user} subscribed to {}", sub.topic());
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let broker = Broker::new(Duration::from_secs(1));
    let roster = Arc::new(Roster::default());
    broker.register("orders", roster.clone());
    broker.register("payments", roster.clone());
    broker.register("admin", Audit);

    let subs = [
        Subscription::new("s1", "orders"),
        Subscription::new("s2", "payments"),
        Subscription::new("s3", "orders"),
        Subscription::new("s4", "admin").with_param("user", "ada"),
        Subscription::new("s5", "admin"),
        Subscription::new("s6", "unknown"),
    ];
    for sub in &subs {
        match broker.subscribe(sub).await {
            Ok(()) => println!("{} on {}: accepted", sub.id(), sub.topic()),
            Err(rejection) => println!("{} on {}: rejected ({rejection})", sub.id(), sub.topic()),
        }
    }

    broker.unsubscribe(&subs[0]).await.unwrap();
    println!(
        "roster after unsubscribing s1: {:?}",
        roster.by_topic.lock().unwrap()
    );
}
