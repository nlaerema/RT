mod app;
mod renderer;

use app::App;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let app = App::default();
    app.run()
}