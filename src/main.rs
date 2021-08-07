use log::info;

use exchange_prototype::core::ExchangeCore;

fn main() {
    env_logger::init();
    info!("Starting Up...");
    let core = ExchangeCore::new();
    core.run();
}
