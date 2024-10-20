// code largely from https://github.com/smol-rs/smol-macros/commit/a6b3174e457d857f476415f6d3f31a242a0e7609

use async_executor::Executor;
use event_listener::Event;
use std::sync::atomic::{AtomicBool, Ordering};

/// Wait for the executor to stop.
pub struct WaitForStop {
    /// Whether or not we need to stop.
    stopped: AtomicBool,

    /// Wait for the stop.
    events: Event,
}

impl WaitForStop {
    /// Create a new wait for stop.
    #[inline]
    pub fn new() -> Self {
        Self {
            stopped: AtomicBool::new(false),
            events: Event::new(),
        }
    }

    /// Wait for the event to stop.
    #[inline]
    pub async fn wait(&self) {
        loop {
            if self.stopped.load(Ordering::Relaxed) {
                return;
            }

            event_listener::listener!(&self.events => listener);

            if self.stopped.load(Ordering::Acquire) {
                return;
            }

            listener.await;
        }
    }

    /// Stop the waiter.
    #[inline]
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.events.notify_additional(usize::MAX);
    }
}

/// Run a function that takes an `Executor` inside of a thread pool.
#[inline]
pub fn with_thread_pool<T>(ex: &Executor<'_>, f: impl FnOnce() -> T) -> T {
    let stopper = WaitForStop::new();

    // Create a thread for each CPU.
    std::thread::scope(|scope| {
        let num_threads = std::thread::available_parallelism().map_or(1, |num| num.get());
        for i in 0..num_threads {
            let ex = &ex;
            let stopper = &stopper;

            std::thread::Builder::new()
                .name(format!("smol-macros-{i}"))
                .spawn_scoped(scope, || {
                    async_io::block_on(ex.run(stopper.wait()));
                })
                .expect("failed to spawn thread");
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

        stopper.stop();

        match result {
            Ok(value) => value,
            Err(err) => std::panic::resume_unwind(err),
        }
    })
}
