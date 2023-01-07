use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// https://github.com/MicrosoftDocs/minecraft-creator/blob/main/creator/Reference/Content/SchemasReference/Schemas/minecraftSchema_actor_animation_1.8.0.md
#[derive(Serialize, Deserialize)]
pub struct AnimationFile {
    pub format_version: String,
    pub animations: Vec<Animation>,
}

#[derive(Serialize, Deserialize)]
pub struct Animation {
    /// should this animation stop, loop, or stay on the last frame when finished (true, false, "hold_on_last_frame"
    #[serde(rename = "loop")]
    pub loop_: AnimationLoop,

    /// How long to wait in seconds before playing this animation.  Note that this expression is evaluated once before playing, and only re-evaluated if asked to play from the beginning again.  A looping animation should use 'loop_delay' if it wants a delay between loops.
    pub start_delay: AnimationBoneValue,

    /// How long to wait in seconds before looping this animation.  Note that this expression is evaluated after each loop and on looping animation only.
    pub loop_delay: AnimationBoneValue,

    /// how does time pass when playing the animation.  Defaults to "query.anim_time + query.delta_time" which means advance in seconds.
    pub anim_time_update: AnimationBoneValue,
    pub blend_weight: AnimationBoneValue,

    /// reset bones in this animation to the default pose before applying this animation
    pub override_previous_animation: bool,
    pub bones: HashMap<String, AnimationBone>,

    /// override calculated value (set as the max keyframe or event time) and set animation length in seconds.
    pub animation_length: f32
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationLoop {
    False,
    True,
    HoldOnLastFrame
}

#[derive(Serialize, Deserialize)]
pub struct AnimationBone {
    pub position: AnimationBoneElement,
    pub rotation: AnimationBoneElement,
    pub scale: AnimationBoneElement
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimationBoneElement {
    NormalSplat(AnimationBoneValue),
    Normal([AnimationBoneValue; 3]),
    /*Keyframe(HashMap<f32, AnimationBoneKeyframe>)*/
}

#[derive(Serialize, Deserialize)]
pub struct AnimationBoneKeyframe {
    pub lerp_mode: String,
    pub pre: [AnimationBoneValue; 3],
    pub post: [AnimationBoneValue; 3]
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimationBoneValue {
    Literal(f32),
    Molang(String)
}
