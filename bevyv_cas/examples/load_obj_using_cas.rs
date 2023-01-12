use bevy::prelude::*;
use bevyv_cas::CasAssetIo;
use bevyv_obj::ObjPlugin;

fn main() {
    App::new()
        .insert_resource(AssetServer::new(CasAssetIo::default()))
        .add_plugins(DefaultPlugins)
        .add_plugin(ObjPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(Camera3dBundle {
        camera: Camera::default(),
        transform: Transform::from_xyz(-5.0, 5.0, 5.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: asset_server.load("capsule.obj"),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load("capsule0.jpg")),
            ..default()
        }),
        ..default()
    });
}
