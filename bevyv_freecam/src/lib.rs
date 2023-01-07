use std::f32::consts::{FRAC_PI_2, PI};

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

#[derive(Component)]
pub struct Freecam;

pub fn grab_mouse(
    mut windows: ResMut<Windows>,
    mouse_input: Res<Input<MouseButton>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    let window = windows.primary_mut();
    if mouse_input.just_pressed(MouseButton::Left) {
        window.set_cursor_visibility(false);
        window.set_cursor_grab_mode(CursorGrabMode::Locked);
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        window.set_cursor_visibility(true);
        window.set_cursor_grab_mode(CursorGrabMode::None);
    }
}

pub fn process_input(
    time: Res<Time>,
    windows: Res<Windows>,
    keyboard_input: Res<Input<KeyCode>>,
    /*mouse_button_input: Res<Input<MouseButton>>,*/
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    /*mut mouse_wheel_event_reader: EventReader<MouseWheel>,
    gamepad_axes: Res<Axis<GamepadAxis>>,*/
    mut query: Query<&mut Transform, With<Freecam>>,
) {
    let mut mouse_delta: Vec2 = Vec2::ZERO;
    if !windows.get_primary().unwrap().cursor_visible() {
        for event in mouse_motion_event_reader.iter() {
            mouse_delta += event.delta;
        }
    }

    let time_delta = time.raw_delta_seconds();

    for mut transform in query.iter_mut() {
        let mut move_x = 0.0;
        let mut move_y = 0.0;
        let mut move_z = 0.0;
        if keyboard_input.pressed(KeyCode::W) {
            move_x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            move_x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            move_y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::LShift) {
            move_y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) {
            move_z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            move_z -= 1.0;
        }
        if move_x != 0.0 || move_y != 0.0 || move_z != 0.0 {
            let move_vec = transform.rotation * Vec3::new(-move_z, 0., -move_x) + Vec3::new(0., move_y, 0.);
            transform.translation += move_vec * time_delta * 20.0;
        }

        if mouse_delta.x.abs() > 1e-5 || mouse_delta.y.abs() > 1e-5 {
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            transform.rotation = Quat::from_euler(EulerRot::YXZ, ((yaw + (mouse_delta.x * -0.0003)) % (PI * 2.0)), (pitch + (mouse_delta.y * -0.0003)).clamp(-FRAC_PI_2, FRAC_PI_2), 0.0);
        }
    }
}
