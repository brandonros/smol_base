use std::sync::Arc;

use simple_error::SimpleResult;
use smol::Executor;
use smol_base::smol_main;

async fn async_main(_ex: &Arc<Executor<'static>>) -> SimpleResult<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    log::info!("Hello, world!");
    Ok(())
}

smol_main!(async_main);
