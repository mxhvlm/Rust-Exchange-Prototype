use exchange_prototype::core::ExchangeCore;

fn main() {
    env_logger::init();
    let core = ExchangeCore::new();
    core.run();
}
