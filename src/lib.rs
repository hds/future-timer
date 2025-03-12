//! Future timing instrumentation.
//!
//! Provides instrumentation to record the time taken by a future. This includes the busy time and
//! the idle time.
//!
//! ## Busy time
//!
//! The busy time of a future is the sum of all the time consumed during calls to [`Future::poll`]
//! on that future.
//!
//! ## Idle time
//!
//! The idle time of a future is the sum of all the time between calls to [`Future::poll`]. The
//! time before the first poll is not included.
//!
//! # Usage
//!
//! First, add this to your Cargo.toml `dependencies`:
//!
//! ```toml
//! future-timing = "0.1"
//! ```
//!
//! Record the timing of a future in the following manner.
//!
//! ```
//! # async fn some_async_fn() -> u64 {
//! #   tokio::time::sleep(std::time::Duration::from_micros(10)).await;
//! #   42
//! # }
//! # fn do_something_with_output(_: u64) {}
//! # #[tokio::main]
//! # async fn main() {
//!     let output = future_timing::timed(some_async_fn()).await;
//!     let (timing, future_output) = output.into_parts();
//!
//!     do_something_with_output(future_output);
//!
//!     assert!(!timing.idle().is_zero());
//!     assert!(!timing.busy().is_zero());
//! # }
//! ```
//!
//! # Comparison with similar crates
//!
//! This is a single purpose crate, created because I couldn't find any other crate that included
//! the functionality I needed, which is to say, record future timing and make it available to the
//! code that awaited the future upon that future resolving.
//!
//! If you want to record and analyze the timing of many different futures (and you're using the
//! [Tokio runtime], then you can use Tokio's [`RuntimeMetrics`] for an aggregated view or [Tokio
//! Console] to see the timings of each task individually.
//!
//! If you don't actualy want to record the timing of a future, but instead want a future which
//! resolves after a specific period of time, then you're in the wrong place. Have a look at the
//! [`async-timer`] crate instead.
//!
//! # Supported Rust Versions
//!
//! `future-timing` is built against the latest stable release. The minimum supported version is
//! 1.70. The current version of `future-timing` is not guaranteed to build on Rust versions earlier
//! than the minimum supported version.
//!
//! # License
//!
//! This project is licensed under the [MIT license].
//!
//! ## Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
//! in `future-timing` by you, shall be licensed as MIT, without any additional terms or conditions.
//!
//! [`async-timer`]: https://docs.rs/async-timer/latest/async_timer/
//! [`RuntimeMetrics`]: https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html
//! [Tokio Console]: https://docs.rs/tokio-console/latest/tokio_console/
//! [Tokio runtime]: https://docs.rs/tokio/latest/tokio/
//! [MIT license]: https://github.com/hds/future-timing/blob/main/LICENSE
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use pin_project_lite::pin_project;

/// Instrument a future to record its timing
///
/// The busy and idle time for the future will be recorded separately in the result together with
/// the output of the wrapped future. See the documentation for [`Timing`] for more details.
///
/// # Examples
///
/// ```
/// # async fn some_async_fn() -> u64 {
/// #   tokio::time::sleep(std::time::Duration::from_micros(10)).await;
/// #   42
/// # }
/// # fn do_something_with_output(_: u64) {}
/// # #[tokio::main]
/// # async fn main() {
///     let output = future_timing::timed(some_async_fn()).await;
///     let (timing, future_output) = output.into_parts();
///
///     do_something_with_output(future_output);
///
///     assert!(!timing.idle().is_zero());
///     assert!(!timing.busy().is_zero());
/// # }
pub fn timed<F>(fut: F) -> Timed<F>
where
    F: Future,
{
    Timed::new(fut)
}

pin_project! {
    /// Instrumentation to record the timing of a wrapped future.
    ///
    /// The `Timed` wraps any future and records the inner future's busy and idle time. The
    /// timing is returned together with the inner future's output once it resolves ready.
    ///
    /// # Examples
    ///
    /// To wrap a future, use the [`timed`] function.
    ///
    /// ```
    /// # async fn some_async_fn() -> u64 {
    /// #   tokio::time::sleep(std::time::Duration::from_micros(10)).await;
    /// #   42
    /// # }
    /// # fn do_something_with_output(_: u64) {}
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let output = future_timing::timed(some_async_fn()).await;
    ///     let (timing, future_output) = output.into_parts();
    ///
    ///     do_something_with_output(future_output);
    ///
    ///     assert!(!timing.idle().is_zero());
    ///     assert!(!timing.busy().is_zero());
    /// # }
    /// ```
    pub struct Timed<F>
    where
        F: Future,
    {
        last_poll_end: Option<Instant>,
        idle: Duration,
        busy: Duration,

        #[pin]
        inner: F,
    }
}

impl<F> Timed<F>
where
    F: Future,
{
    fn new(inner: F) -> Self {
        Self {
            last_poll_end: None,
            idle: Duration::ZERO,
            busy: Duration::ZERO,

            inner,
        }
    }
}

impl<F> Future for Timed<F>
where
    F: Future,
{
    type Output = TimingOutput<F::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<TimingOutput<F::Output>> {
        let start = Instant::now();
        let mut this = self.project();
        let result = this.inner.as_mut().poll(cx);
        let end = Instant::now();

        if let Some(last_poll_end) = this.last_poll_end.take() {
            *this.idle += start - last_poll_end;
        }
        *this.busy += end - start;
        *this.last_poll_end = Some(end);

        match result {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready(TimingOutput {
                timing: Timing {
                    idle: *this.idle,
                    busy: *this.busy,
                },
                inner: output,
            }),
        }
    }
}

/// A wrapper around timing information for an instrumented future and that future's output.
///
/// See the documentation on [`Timing`] for further details.
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct TimingOutput<T> {
    timing: Timing,
    inner: T,
}

impl<T> TimingOutput<T> {
    /// Returns the timing of the future that was instrumented.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn some_async_fn() {}
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let output = future_timing::timed(some_async_fn()).await;
    ///     let timing: future_timing::Timing = output.timing();
    /// #   _ = timing
    /// # }
    /// ```
    #[must_use]
    pub fn timing(&self) -> Timing {
        self.timing
    }

    /// Returns the timing of the future and its output.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn some_async_fn() {}
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let output = future_timing::timed(some_async_fn()).await;
    ///     let (timing, future_output) = output.into_parts();
    /// #   _ = timing;
    /// #   _ = future_output;
    /// # }
    /// ```
    #[must_use]
    pub fn into_parts(self) -> (Timing, T) {
        (self.timing, self.inner)
    }

    /// Returns the future's output.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn some_async_fn() {}
    /// # #[tokio::main]
    /// # async fn main() {
    ///     let output = future_timing::timed(some_async_fn()).await;
    ///     let future_output = output.into_inner();
    /// #   _ = future_output;
    /// # }
    /// ```
    #[must_use]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

/// The timing information for an instrumented future.
///
/// The busy time and the idle time of the instrumented future is available.
///
/// ## Busy time
///
/// The busy time of a future is the sum of all the time consumed during calls to [`Future::poll`]
/// on that future.
///
/// The busy time will always be non-zero.
///
/// ## Idle time
///
/// The idle time of a future is the sum of all the time between calls to [`Future::poll`]. The
/// time before the first poll is not included.
///
/// The idle time may be zero if the inner future returns [`Poll::Ready`] on the first poll (and so
/// never returns [`Poll::Pending`]).
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub struct Timing {
    idle: Duration,
    busy: Duration,
}

impl Timing {
    /// The sum of all poll durations.
    ///
    /// This is the total time the future was polled across all polls.
    #[must_use]
    pub fn busy(&self) -> Duration {
        self.busy
    }

    /// The sum of all durations between polls.
    ///
    /// This is the total time between each poll of the timed future. The time before the first
    /// poll and after the final poll is not included.
    #[must_use]
    pub fn idle(&self) -> Duration {
        self.idle
    }
}
