use bevy::prelude::*;
use rodio::cpal::traits::HostTrait;
use rodio::dynamic_mixer::{mixer, DynamicMixerController};
use rodio::{cpal, queue, DeviceTrait, OutputStream, Source};
use std::sync::{Arc, Mutex};
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
        let hrtf = Hrtf::new(
            &context,
            device.default_output_config().unwrap().sample_rate().0,
            512,
            HrtfType::Default,
        )
        .unwrap();
        let scene = steamaudio::simulation::Scene::new(&context).unwrap();
        let simulator = Simulator::new(
            &context,
            device.default_output_config().unwrap().sample_rate().0,
            512,
            &scene,
        )
        .unwrap();

        // Create Ambisonics submix
        let (mixer_controller, mixer) = mixer(
            (2 as u16 + 1).pow(2),
            device.default_output_config().unwrap().sample_rate().0,
        );
        let listener_args = Arc::new(Mutex::new(ListenerArgs {
            position: Default::default(),
            rotation: Default::default(),
        }));
        let listener_args2 = listener_args.clone();
        let mut ambisonics_decode = AmbisonicsDecodeEffect::new(
            &context,
            device.default_output_config().unwrap().sample_rate().0,
            512,
            SpeakerLayout::Stereo,
            &hrtf,
            2,
        )
        .unwrap();
        queue_tx.append(steamaudio::transform::transform(
            mixer,
            move |in_, out| {
                ambisonics_decode.apply(listener_args2.lock().unwrap().rotation, 2, true, in_, out);
            },
            2,
            512,
        ));

        app.insert_resource(AudioResources {
            context,
            hrtf,
            scene,
            simulator,
            listener_args,
            mixer_controller,
        })
        .add_system(update_sounds)
        .add_system(update_listener)
        .add_system(run_simulator);
    }
}

#[derive(Component)]
pub struct Listener;

#[derive(Component)]
pub struct AirAbsorption ;

#[derive(Bundle)]
pub struct SoundBundle {
    pub source: Handle<AudioSource>,

    pub transform: Transform,
}

#[derive(Resource)]
struct AudioResources {
    context: Context,
    hrtf: Hrtf,
    scene: steamaudio::simulation::Scene,
    simulator: Simulator,

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
    simulation_source: steamaudio::simulation::Source,

    position: Vec3,
}

fn update_sounds(
    mut commands: Commands,
    audio_resources: Res<AudioResources>,
    audio_sources: Res<Assets<AudioSource>>,
    uninitialized_sounds: Query<(Entity, &Handle<AudioSource>, &Transform), Without<Sound>>,
    mut sounds: Query<(&mut Sound, &Transform), Changed<Transform>>,
) {
    for (entity, sound, transform) in uninitialized_sounds.iter() {
        if let Some(audio_source) = audio_sources.get(sound) {
            let audio_source = audio_source.decoder().convert_samples();
            let audio_source_channels = audio_source.channels();

            let sound = Sound {
                args: Arc::new(Mutex::new(SoundArgs {
                    simulation_source: steamaudio::simulation::Source::new(&audio_resources.simulator)
                        .unwrap(),
                    position: transform.translation,
                })),
            };
            audio_resources.simulator.commit();

            let mut direct_effect = DirectEffect::new(
                &audio_resources.context,
                audio_source.sample_rate(),
                512,
                audio_source_channels,
            )
            .unwrap();
            let mut ambisonics_encode = AmbisonicsEncodeEffect::new(
                &audio_resources.context,
                audio_source.sample_rate(),
                512,
                2,
            )
            .unwrap();

            let sound_args = sound.args.clone();
            let sound_args2 = sound.args.clone();
            let listener_args = audio_resources.listener_args.clone();
            audio_resources
                .mixer_controller
                .add(steamaudio::transform::transform(
                    steamaudio::transform::transform(
                        audio_source,
                        move |in_, out| {
                            direct_effect.apply(&sound_args.lock().unwrap().simulation_source, in_, out);
                        },
                        audio_source_channels,
                        512,
                    ),
                    move |in_, out| {
                        ambisonics_encode.apply(
                            (listener_args.lock().unwrap().position
                                - sound_args2.lock().unwrap().position)
                                .normalize(),
                            2,
                            in_,
                            out,
                        );
                    },
                    (2 as u16 + 1).pow(2),
                    512,
                ));
            commands.entity(entity).insert(sound);
        }
    }

    for (sound, transform) in sounds.iter_mut() {
        let mut sound_args = sound.args.lock().unwrap();
        sound_args.simulation_source.update(Orientation {
            translation: transform.translation,
            rotation: transform.rotation,
        });
        sound_args.position = transform.translation;
    }
}

fn update_listener(
    mut audio_resources: ResMut<AudioResources>,
    listener: Query<&Transform, (With<Listener>, Changed<Transform>)>,
) {
    if !listener.is_empty() {
        let transform = listener.single();

        audio_resources.simulator.update(Orientation {
            translation: transform.translation,
            rotation: transform.rotation,
        });

        let mut listener_args = audio_resources.listener_args.lock().unwrap();
        listener_args.position = transform.translation;
        listener_args.rotation = transform.rotation;
    }
}

fn run_simulator(mut audio_resources: ResMut<AudioResources>) {
    audio_resources.simulator.run_direct()
}
