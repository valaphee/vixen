use std::f32::consts::{FRAC_PI_2, PI};

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;

use vixen_steamaudio::{AudioPlugin, Listener, SoundBundle};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AudioPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(grab_mouse)
        .add_system(process_input)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: Res<AssetServer>,
    mut animations: ResMut<Assets<AnimationClip>>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0), /*.looking_at(Vec3::ZERO, Vec3::Y)*/
            ..default()
        },
        Freecam,
        Listener,
    ));

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 200.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    let animation_marker = Name::new("marker");
    let mut animation = AnimationClip::default();
    animation.add_curve_to_path(
        EntityPath {
            parts: vec![animation_marker.clone()],
        },
        VariableCurve {
            keyframe_timestamps: vec![0.0, 1.0, 2.0, 3.0, 4.0],
            keyframes: Keyframes::Translation(vec![
                Vec3::new(50.0, 0.0, 50.0),
                Vec3::new(-50.0, 0.0, 50.0),
                Vec3::new(-50.0, 0.0, -50.0),
                Vec3::new(50.0, 0.0, -50.0),
                Vec3::new(50.0, 0.0, 50.0),
            ]),
        },
    );

    let mut animation_player = AnimationPlayer::default();
    animation_player.play(animations.add(animation)).repeat();

    commands.spawn((
        SoundBundle {
            source: asset_server.load("Windless Slopes.ogg"),
            transform: Transform::from_xyz(1.0, 0.0, 0.0),
        },
        animation_marker,
        animation_player,
        meshes.add(Mesh::from(shape::Cube { size: 5.0 })),
        materials.add(Color::rgb(0.8, 0.0, 0.6).into()),
        GlobalTransform::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    ));
}

////////////////////////////////////////////
#[derive(Component)]
pub struct Freecam;

fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    if !windows.is_empty() {
        let mut window = windows.single_mut();

        if mouse.just_pressed(MouseButton::Left) {
            window.cursor.visible = false;
            window.cursor.grab_mode = CursorGrabMode::Locked;
        }

        if key.just_pressed(KeyCode::Escape) {
            window.cursor.visible = true;
            window.cursor.grab_mode = CursorGrabMode::None;
        }
    }
}

pub fn process_input(
    time: Res<Time>,
    mut windows: Query<&Window>,
    keyboard_input: Res<Input<KeyCode>>,
    /*mouse_button_input: Res<Input<MouseButton>>,*/
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    /*mut mouse_wheel_event_reader: EventReader<MouseWheel>,
    gamepad_axes: Res<Axis<GamepadAxis>>,*/
    mut query: Query<&mut Transform, With<Freecam>>,
) {
    let mut mouse_delta: Vec2 = Vec2::ZERO;
    if !windows.is_empty() {
        if !windows.single_mut().cursor.visible {
            for event in mouse_motion_event_reader.iter() {
                mouse_delta += event.delta;
            }
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
            let move_vec =
                transform.rotation * Vec3::new(-move_z, 0., -move_x) + Vec3::new(0., move_y, 0.);
            transform.translation += move_vec * time_delta * 20.0;
        }

        if mouse_delta.x.abs() > 1e-5 || mouse_delta.y.abs() > 1e-5 {
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                ((yaw + (mouse_delta.x * -0.0003)) % (PI * 2.0)),
                (pitch + (mouse_delta.y * -0.0003)).clamp(-FRAC_PI_2, FRAC_PI_2),
                0.0,
            );
        }
    }
}
