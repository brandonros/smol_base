pub mod smol_thread_pool;
pub use smol_thread_pool::*;

#[macro_export]
macro_rules! smol_main {
    ($main_func:expr) => {
        fn main() -> SimpleResult<()> {
            // create executor
            let ex = Arc::new(Executor::new());

            // run executor on thread pool
            $crate::with_thread_pool(&ex, || async_io::block_on($main_func(&ex)))
        }
    };
}
