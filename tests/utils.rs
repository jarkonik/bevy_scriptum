use bevy::{app::App, asset::AssetPlugin, core::TaskPoolPlugin, ecs::system::Resource};
use bevy_scriptum::ScriptingPlugin;

pub fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((AssetPlugin::default(), TaskPoolPlugin::default()))
        .add_plugins(ScriptingPlugin);
    app.update();
    app
}

pub fn run_scripting_with(app: &mut App, f: impl FnOnce(&mut App)) {
    app.update(); // Execute plugin internal systems
    f(app);
    app.update(); // Execute systems added by callback
}

#[derive(Default, Resource)]
pub struct TimesCalled {
    pub times_called: u8,
}
