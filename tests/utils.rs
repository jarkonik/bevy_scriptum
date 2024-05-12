use bevy::{
    app::App,
    ecs::{component::Component, system::Resource},
};


#[derive(Component, Default)]
struct MyCompnent;

pub fn run_scripting_with(app: &mut App, f: impl FnOnce(&mut App)) {
    app.update(); // Execute plugin internal systems
    f(app);
    app.update(); // Execute systems added by callback
}

#[derive(Default, Resource)]
pub struct TimesCalled {
    pub times_called: u8,
}
