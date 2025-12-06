#![allow(dead_code, unused_imports)]

use std::panic;
use std::panic::UnwindSafe;

/// Convert any panic payload into a readable String
fn panic_message(err: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        // fallback to Debug
        format!("{:?}", err)
    }
}

fn might_panic() {
    println!("About to panic!");
    panic!("Something went wrong!");
}

#[test]
fn test_panic_handler() {
    let result = panic::catch_unwind(|| {
        might_panic();
    });

    if let Err(err) = result {
        assert_eq!(panic_message(&*err), "Something went wrong!");
    };
}
