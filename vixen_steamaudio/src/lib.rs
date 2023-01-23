use bevy::prelude::*;
use rodio::dynamic_mixer::{mixer, DynamicMixerController};
use rodio::{cpal, DeviceTrait, OutputStream, queue, Source};
use std::sync::{Arc, Mutex};
use rodio::cpal::traits::HostTrait;
use steamaudio::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        // Get default output device, and leak stream to keep the device open
        let device = cpal::default_host().default_output_device().unwrap();
        let (stream, handle) = OutputStream::try_from_device(&device).unwrap();
        std::mem::forget(stream);

        // Create source queue
        let (queue_tx, queue_rx) = queue::queue(true);
        handle.play_raw(queue_rx).unwrap();

        // Initialize SteamAudio
        let context = Context::default();
        let hrtf = Hrtf::new(&context, device.default_output_config().unwrap().sample_rate().0, 512, HrtfType::Default).unwrap();
        let mut effect = AmbisonicsDecodeEffect::new(&context, device.default_output_config().unwrap().sample_rate().0, 512, SpeakerLayout::Stereo, &hrtf, 2).unwrap();

        // Create Ambisonics submix
        let listener_args = Arc::new(Mutex::new(ListenerArgs {
            position: Default::default(),
            rotation: Default::default(),
        }));
        let listener_args2 = listener_args.clone();
        let (mixer_controller, mixer) = mixer((2 as u16 + 1).pow(2), device.default_output_config().unwrap().sample_rate().0);
        queue_tx.append(steamaudio::transform::transform(
            mixer,
            move |in_, out| {
                effect.apply(
                    listener_args2.lock().unwrap().rotation,
                    2,
                    true,
                    in_,
                    out,
                );
            },
            2,
            512,
        ));

        app.insert_resource(AudioResources {
            context,
            hrtf,
            listener_args,
            mixer_controller,
        })
        .add_system(update_sounds)
        .add_system(update_listener);
    }
}

#[derive(Component)]
pub struct Listener;

#[derive(Bundle)]
pub struct SoundBundle {
    pub sound: Handle<AudioSource>,
    pub transform: Transform,
}

#[derive(Resource)]
struct AudioResources {
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
struct Sound {
    args: Arc<Mutex<SoundArgs>>,
}

struct SoundArgs {
    position: Vec3,
}

fn update_sounds(
    mut commands: Commands,
    audio: Res<AudioResources>,
    audio_sources: Res<Assets<AudioSource>>,
    uninitialized_sounds: Query<(Entity, &Handle<AudioSource>, &Transform), Without<Sound>>,
    mut sounds: Query<(&mut Sound, &Transform), Changed<Transform>>,
) {
    for (entity, sound, transform) in uninitialized_sounds.iter() {
        if let Some(audio_source) = audio_sources.get(sound) {
            let audio_source = audio_source.decoder().convert_samples();
            let mut effect = AmbisonicsEncodeEffect::new(&audio.context, audio_source.sample_rate(), 512, 2).unwrap();
            let sound = Sound {
                args: Arc::new(Mutex::new(SoundArgs {
                    position: transform.translation,
                })),
            };

            let sound_args = sound.args.clone();
            let listener_args = audio.listener_args.clone();
            audio.mixer_controller.add(steamaudio::transform::transform(
                audio_source,
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
                (2 as u16 + 1).pow(2),
                512,
            ));
            commands.entity(entity).insert(sound);
        }
    }

    for (sound, transform) in sounds.iter_mut() {
        sound.args.lock().unwrap().position = transform.translation;
    }
}

fn update_listener(audio: Res<AudioResources>, listener: Query<&Transform, (With<Listener>, Changed<Transform>)>) {
    if !listener.is_empty() {
        let transform = listener.single();
        let mut listener_args = audio.listener_args.lock().unwrap();
        listener_args.position = transform.translation;
        listener_args.rotation = transform.rotation;
    }
}
