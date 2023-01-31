use bevy::prelude::*;
use prometheus::guid::Guid;
use vixen_prometheus::texture::TeTextureLoader;
use vixen_prometheus::asset_io::AssetIo;

fn main() {
    App::new()
        .insert_resource(AssetServer::new(AssetIo::default()))
        .add_plugins(DefaultPlugins)
        .init_asset_loader::<TeTextureLoader>()
        .add_startup_system(setup)
        .add_system(change_display)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let guid = Guid {
        engine: 0,
        type_: 4,
        platform: 0,
        region: 0,
        locale: 0,
        index: 1,
    };
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(format!("{}.004", guid.to_raw())),
            ..default()
        },
        Display { index: 0 },
    ));
}

#[derive(Component)]
struct Display {
    index: u32,
}

fn change_display(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut show_entities: Query<(Entity, &mut Display)>,
) {
    if input.just_pressed(KeyCode::Q) {
        let (entity, mut show_component) = show_entities.single_mut();
        show_component.index += 1;
        println!("{}", show_component.index);

        let guid = Guid {
            engine: 0,
            type_: 4,
            platform: 0,
            region: 0,
            locale: 0,
            index: show_component.index,
        };
        let image: Handle<Image> = asset_server.load(format!("{}.004", guid.to_raw()));
        commands.entity(entity).insert(image);
    }
    if input.just_pressed(KeyCode::W) {
        let (entity, mut show_component) = show_entities.single_mut();
        show_component.index += 10;
        println!("{}", show_component.index);

        let guid = Guid {
            engine: 0,
            type_: 4,
            platform: 0,
            region: 0,
            locale: 0,
            index: show_component.index,
        };
        let image: Handle<Image> = asset_server.load(format!("{}.004", guid.to_raw()));
        commands.entity(entity).insert(image);
    }
    if input.just_pressed(KeyCode::E) {
        let (entity, mut show_component) = show_entities.single_mut();
        show_component.index += 100;
        println!("{}", show_component.index);

        let guid = Guid {
            engine: 0,
            type_: 4,
            platform: 0,
            region: 0,
            locale: 0,
            index: show_component.index,
        };
        let image: Handle<Image> = asset_server.load(format!("{}.004", guid.to_raw()));
        commands.entity(entity).insert(image);
    }
    if input.just_pressed(KeyCode::R) {
        let (entity, mut show_component) = show_entities.single_mut();
        show_component.index += 1000;
        println!("{}", show_component.index);

        let guid = Guid {
            engine: 0,
            type_: 4,
            platform: 0,
            region: 0,
            locale: 0,
            index: show_component.index,
        };
        let image: Handle<Image> = asset_server.load(format!("{}.004", guid.to_raw()));
        commands.entity(entity).insert(image);
    }

    if input.just_pressed(KeyCode::F) {
        let (entity, mut show_component) = show_entities.single_mut();
        show_component.index -= 1000;
        println!("{}", show_component.index);

        let guid = Guid {
            engine: 0,
            type_: 4,
            platform: 0,
            region: 0,
            locale: 0,
            index: show_component.index,
        };
        let image: Handle<Image> = asset_server.load(format!("{}.004", guid.to_raw()));
        commands.entity(entity).insert(image);
    }
    if input.just_pressed(KeyCode::D) {
        let (entity, mut show_component) = show_entities.single_mut();
        show_component.index -= 100;
        println!("{}", show_component.index);

        let guid = Guid {
            engine: 0,
            type_: 4,
            platform: 0,
            region: 0,
            locale: 0,
            index: show_component.index,
        };
        let image: Handle<Image> = asset_server.load(format!("{}.004", guid.to_raw()));
        commands.entity(entity).insert(image);
    }
    if input.just_pressed(KeyCode::S) {
        let (entity, mut show_component) = show_entities.single_mut();
        show_component.index -= 10;
        println!("{}", show_component.index);

        let guid = Guid {
            engine: 0,
            type_: 4,
            platform: 0,
            region: 0,
            locale: 0,
            index: show_component.index,
        };
        let image: Handle<Image> = asset_server.load(format!("{}.004", guid.to_raw()));
        commands.entity(entity).insert(image);
    }
    if input.just_pressed(KeyCode::A) {
        let (entity, mut show_component) = show_entities.single_mut();
        show_component.index -= 1;
        println!("{}", show_component.index);

        let guid = Guid {
            engine: 0,
            type_: 4,
            platform: 0,
            region: 0,
            locale: 0,
            index: show_component.index,
        };
        let image: Handle<Image> = asset_server.load(format!("{}.004", guid.to_raw()));
        commands.entity(entity).insert(image);
    }
}
