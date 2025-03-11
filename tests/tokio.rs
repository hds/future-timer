use std::time::Duration;

use future_timer::FutureTimer;

#[tokio::test]
async fn never_yield() {
    let output = FutureTimer::new(async { 42 }).await;
    let timing = output.timing();

    assert!(timing.idle().is_zero());
    assert!(!timing.busy().is_zero());
    assert_eq!(output.into_inner(), 42);
}

#[tokio::test]
async fn short_async_sleep() {
    let output = FutureTimer::new(async {
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
    let output = FutureTimer::new(async {
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
