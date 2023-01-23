use bevy::prelude::*;
use rodio::dynamic_mixer::{mixer, DynamicMixerController};
use rodio::{queue, Source};
use std::sync::{Arc, Mutex};
use steamaudio::prelude::*;

pub mod transform;

pub struct SteamAudioPlugin;

impl Plugin for SteamAudioPlugin {
    fn build(&self, app: &mut App) {
        let context = Context::default();
        let hrtf = Hrtf::new(&context, 44100, 512, HrtfType::Default).unwrap();

        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        std::mem::forget(stream);

        let (queue_tx, queue_rx) = queue::queue(true);
        handle.play_raw(queue_rx).unwrap();

        let (mixer_controller, mixer) = mixer(9, 44100);

        let mut effect =
            AmbisonicsDecodeEffect::new(&context, 44100, 512, SpeakerLayout::Stereo, &hrtf, 2)
                .unwrap();
        let listener_args = Arc::new(Mutex::new(ListenerArgs {
            position: Default::default(),
            rotation: Default::default(),
        }));
        let listener_args_move = listener_args.clone();
        queue_tx.append(transform::transform(
            mixer,
            move |in_, out| {
                effect.apply(
                    listener_args_move.lock().unwrap().rotation,
                    2,
                    true,
                    in_,
                    out,
                );
            },
            2,
            512,
        ));

        app.insert_resource(Audio {
            context,
            hrtf,
            listener_args,
            mixer_controller,
        })
        .add_system(update_sounds)
        .add_system(update_listener);
    }
}

#[derive(Resource)]
struct Audio {
    context: Context,
    hrtf: Hrtf,

    listener_args: Arc<Mutex<ListenerArgs>>,

    mixer_controller: Arc<DynamicMixerController<f32>>,
}

struct ListenerArgs {
    position: Vec3,
    rotation: Quat,
}

#[derive(Component)]
pub struct Listener;

#[derive(Bundle)]
pub struct SoundBundle {
    pub sound: Handle<AudioSource>,
    pub transform: Transform,
}

#[derive(Component)]
struct Sound {
    args: Arc<Mutex<SoundArgs>>,
}

struct SoundArgs {
    position: Vec3,
}

fn update_sounds(
    mut commands: Commands,
    audio: Res<Audio>,
    audio_sources: Res<Assets<AudioSource>>,
    uninitialized_sounds: Query<(Entity, &Handle<AudioSource>, &Transform), Without<Sound>>,
    mut sounds: Query<(&mut Sound, &Transform), Changed<Transform>>,
) {
    for (entity, sound, transform) in uninitialized_sounds.iter() {
        if let Some(audio_source) = audio_sources.get(sound) {
            let mut effect = AmbisonicsEncodeEffect::new(&audio.context, 44100, 512, 2).unwrap();
            let sound = Sound {
                args: Arc::new(Mutex::new(SoundArgs {
                    position: transform.translation,
                })),
            };

            let sound_args = sound.args.clone();
            let listener_args = audio.listener_args.clone();
            audio.mixer_controller.add(transform::transform(
                audio_source.decoder().convert_samples(),
                move |in_, out| {
                    effect.apply(
                        (listener_args.lock().unwrap().position
                            - sound_args.lock().unwrap().position)
                            .normalize(),
                        2,
                        in_,
                        out,
                    )
                },
                9,
                512,
            ));
            commands.entity(entity).insert(sound);
        }
    }

    for (sound, transform) in sounds.iter_mut() {
        sound.args.lock().unwrap().position = transform.translation;
    }
}

fn update_listener(audio: Res<Audio>, listener: Query<&Transform, With<Listener>>) {
    let transform = listener.single();
    let mut listener_args = audio.listener_args.lock().unwrap();
    listener_args.position = transform.translation;
    listener_args.rotation = transform.rotation;
}
