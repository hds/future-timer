# future-timing

Future timing instrumentation.

Provides instrumentation to record the time taken by a future. This includes the busy time and
the idle time.

## Busy time

The busy time of a future is the sum of all the time consumed during calls to [`Future::poll`]
on that future.

## Idle time

The idle time of a future is the sum of all the time between calls to [`Future::poll`]. The
time before the first poll is not included.

# Usage

First, add this to your Cargo.toml `dependencies`:

```toml
future-timing = "0.1"
```

Record the timing of a future in the following manner.

```
let output = future_timing::timed(some_async_fn()).await;
let (timing, future_output) = output.into_parts();

do_something_with_output(future_output);

assert!(!timing.idle().is_zero());
assert!(!timing.busy().is_zero());
```

# Comparison with similar crates

This is a single purpose crate, created because I couldn't find any other crate that included
the functionality I needed, which is to say, record future timing and make it available to the
code that awaited the future upon that future resolving.

If you want to record and analyze the timing of many different futures (and you're using the
[Tokio runtime], then you can use Tokio's [`RuntimeMetrics`] for an aggregated view or [Tokio
Console] to see the timings of each task individually.

If you don't actualy want to record the timing of a future, but instead want a future which
resolves after a specific period of time, then you're in the wrong place. Have a look at the
[`async-timer`] crate instead.

# Supported Rust Versions

`future-timing` is built against the latest stable release. The minimum supported version is
1.70. The current version of `future-timing` is not guaranteed to build on Rust versions earlier
than the minimum supported version.

# License

This project is licensed under the [MIT license].

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in `future-timing` by you, shall be licensed as MIT, without any additional terms or conditions.

[`async-timer`]: https://docs.rs/async-timer/latest/async_timer/
['Future::poll`]: https://doc.rust-lang.org/std/future/trait.Future.html#tymethod.poll
[`RuntimeMetrics`]: https://docs.rs/tokio/latest/tokio/runtime/struct.RuntimeMetrics.html
[Tokio Console]: https://docs.rs/tokio-console/latest/tokio_console/
[Tokio runtime]: https://docs.rs/tokio/latest/tokio/
[MIT license]: https://github.com/hds/future-timing/blob/main/LICENSE
