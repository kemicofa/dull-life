use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution, WindowTheme},
};

const WIDTH_RATIO: f32 = 16.0;
const HEIGHT_RATIO: f32 = 9.0;

const MIN_WIDTH: f32 = 1200.0;
const MIN_HEIGHT: f32 = MIN_WIDTH * HEIGHT_RATIO / WIDTH_RATIO;

fn sync_dimensions(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    let width = window.resolution.width();
    let current_height = window.resolution.height();

    let next_height = width * HEIGHT_RATIO / WIDTH_RATIO;

    if current_height == next_height {
        return;
    }

    window.resolution.set(width, next_height);
}

pub struct DullLifeWindowPlugin;

impl Plugin for DullLifeWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dull Life".into(),
                name: Some("dulllife.app".into()),
                resolution:
                    WindowResolution::new(MIN_WIDTH, MIN_HEIGHT).with_scale_factor_override(1.0),
                present_mode: PresentMode::AutoVsync,
                // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, (sync_dimensions, bevy::window::close_on_esc));
    }
}
