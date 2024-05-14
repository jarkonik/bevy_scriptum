use bevy::{
    asset::Asset,
    ecs::{schedule::ScheduleLabel, system::Resource},
    reflect::TypePath,
};
use serde::Deserialize;

use crate::{assets::GetExtensions, Runtime};

/// A script that can be loaded by the [crate::ScriptingPlugin].
#[derive(Asset, Debug, Deserialize, TypePath)]
pub struct RhaiScript(pub String);

impl GetExtensions for RhaiScript {
    fn extensions() -> &'static [&'static str] {
        &["rhai"]
    }
}

impl From<String> for RhaiScript {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Resource)]
pub struct RhaiScriptingRuntime;

#[derive(ScheduleLabel, Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct RhaiSchedule;

impl Runtime for RhaiScriptingRuntime {
    type Schedule = RhaiSchedule;
    type ScriptAsset = RhaiScript;
}
