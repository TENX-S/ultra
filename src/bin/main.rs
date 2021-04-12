use ultra::{
    Launch,
    app::App,
    cli::CLI,
    error::Result,
    config::Config,
    utils::panic_hook,
};

fn main() -> Result<()> {
    // Set up panic hook.
    std::panic::set_hook(Box::new(|info| panic_hook(info)));
    // Get config
    let config = Config::load_sys()?.merge(CLI::new().into())?;

    App::default().bootstrap(&config)?;

    Ok(())
}
