use exchange_prototype::core::ExchangeCore;
use log::info;

fn main() {
    env_logger::init();
    info!("Starting Up...");
    let core = ExchangeCore::new();
    core.run();
}
