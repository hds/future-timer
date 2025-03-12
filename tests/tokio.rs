//! Integration tests running on tokio runtime
//!
//! This crate isn't Tokio specific, and there's no reason why the timing functionality can't be
//! used on a different runtime, but for the purpose of testing, this is the easiest way to get
//! tests running.
use std::time::Duration;

use future_timing::timed;

#[tokio::test]
async fn never_yield() {
    let output = timed(async { 42 }).await;
    let timing = output.timing();

    assert!(timing.idle().is_zero());
    assert!(!timing.busy().is_zero());
    assert_eq!(output.into_inner(), 42);
}

#[tokio::test]
async fn short_async_sleep() {
    let output = timed(async {
        tokio::time::sleep(Duration::from_micros(10)).await;
        42
    })
    .await;
    let timing = output.timing();

    assert!(timing.idle() > Duration::from_micros(10));
    assert!(!timing.busy().is_zero());
    assert_eq!(output.into_inner(), 42);
}

#[tokio::test]
async fn more_busy_time() {
    let output = timed(async {
        std::thread::sleep(Duration::from_micros(200));

        tokio::time::sleep(Duration::from_micros(10)).await;
        42
    })
    .await;
    let timing = output.timing();

    assert!(timing.idle() > Duration::from_micros(10));
    assert!(timing.busy() > Duration::from_micros(200));
    assert_eq!(output.into_inner(), 42);
}
