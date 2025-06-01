use super::{storage::Storage, Handle, Resource};
use std::{error::Error, fmt};

use gltf::Gltf;

#[derive(Debug, Clone)] 
pub enum Keyframes {
    Translation(Vec<Vec<f32>>),
    Other,
}

#[derive(Debug, Clone)] 
pub struct Animation {
    pub name: String,
    pub keyframes: Keyframes,
    pub timestamps: Vec<f32>,
}

impl Resource for Animation {
    type Key = super::TextureKey;
    
    type LoadParams = Animation;
    
    fn load(anim: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>>{
        Ok(Self {
            name: anim.name,
            keyframes: anim.keyframes,
            timestamps: anim.timestamps,
        })
    }
}

#[derive(Debug)]
pub struct LoadAnimationError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl fmt::Display for LoadAnimationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buffer loading error: {}", self.message)
    }
}

impl Error for LoadAnimationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl LoadAnimationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    pub fn with_source(message: impl Into<String>, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

pub type AnimationStorage = Storage<Animation>;
pub type AnimationHandle = Handle<Animation>;


impl Animation {

    pub fn from_gltf_animation_chanel(
        channel: gltf::animation::Channel,
        buffers: &[Vec<u8>], 
        name: String,
    )-> Result<Animation, LoadAnimationError>{
        let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
        let timestamps = if let Some(inputs) = reader.read_inputs() {
            match inputs {
                gltf::accessor::Iter::Standard(times) => {
                    let times: Vec<f32> = times.collect();
                    println!("Time: {}", times.len());
                    // dbg!(times);
                    times
                }
                gltf::accessor::Iter::Sparse(_) => {
                    println!("Sparse keyframes not supported");
                    let times: Vec<f32> = Vec::new();
                    times
                }
            }
        } else {
            println!("We got problems");
            let times: Vec<f32> = Vec::new();
            times
        };
        let keyframes = if let Some(outputs) = reader.read_outputs() {
            match outputs {
                gltf::animation::util::ReadOutputs::Translations(translation) => {
                    let translation_vec = translation.map(|tr| {
                        // println!("Translation:");
                        // dbg!(tr);
                        let vector: Vec<f32> = tr.into();
                        vector
                    }).collect();
                    Keyframes::Translation(translation_vec)
                },
                other => {
                    Keyframes::Other
                }
                // gltf::animation::util::ReadOutputs::Rotations(_) => todo!(),
                // gltf::animation::util::ReadOutputs::Scales(_) => todo!(),
                // gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => todo!(),
            }
        } else {
            println!("We got problems");
            Keyframes::Other
        };
        Ok(Animation {
            name: name.to_string(),
            keyframes,
            timestamps,
        })
    }
    
    pub fn from_gltf_animation(
        gltf_animation: gltf::Animation,
        buffers: &[Vec<u8>], 
    )->Result<Vec<Animation>, LoadAnimationError>{
        let mut animation_clips = Vec::new();
        for channel in gltf_animation.channels() {
            animation_clips.push( match  Animation::from_gltf_animation_chanel(
                channel, 
                buffers,
                gltf_animation.name().unwrap_or("Default").to_string()){
                    Ok(anim) => anim,
                    Err(_) => return Err(LoadAnimationError::new("Error loading animations")),
                }
            ) 
        }
        Ok(animation_clips)
    }

}

pub fn load_gltf_animations(
    gltf: &Gltf,
    buffers: &[Vec<u8>], 
)-> Result<Vec<Animation>, LoadAnimationError>{
    let mut animation_clips = Vec::new();
    for animation in gltf.animations() {
        let mut anims = match Animation::from_gltf_animation(animation, buffers){
                Ok(anims) => anims,
                Err(_) => return Err(LoadAnimationError::new("Error loading animations")),
            };
        animation_clips.append(&mut anims);
    }
    Ok(animation_clips)
}

// pub fn load_gltf_animations(
//     gltf: &Gltf,
//     buffers: &[Vec<u8>], 
// )-> Result<Vec<Animation>, LoadAnimationError>{
//     let mut animation_clips = Vec::new();
//     for animation in gltf.animations() {
//         for channel in animation.channels() {
//             let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));
//             let timestamps = if let Some(inputs) = reader.read_inputs() {
//                 match inputs {
//                     gltf::accessor::Iter::Standard(times) => {
//                         let times: Vec<f32> = times.collect();
//                         times
//                     }
//                     gltf::accessor::Iter::Sparse(_) => {
//                         let times: Vec<f32> = Vec::new();
//                         times
//                     }
//                 }
//             } else {
//                 let times: Vec<f32> = Vec::new();
//                 times
//             };

//             let keyframes = if let Some(outputs) = reader.read_outputs() {
//                 match outputs {
//                     gltf::animation::util::ReadOutputs::Translations(translation) => {
//                         let translation_vec = translation.map(|tr| {
//                             let vector: Vec<f32> = tr.into();
//                             vector
//                         }).collect();
//                         Keyframes::Translation(translation_vec)
//                     },
//                     other => {
//                         Keyframes::Other
//                     }
//                     // gltf::animation::util::ReadOutputs::Rotations(_) => todo!(),
//                     // gltf::animation::util::ReadOutputs::Scales(_) => todo!(),
//                     // gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => todo!(),
//                 }
//             } else {
//                 Keyframes::Other
//             };

//             animation_clips.push(Animation {
//                 name: animation.name().unwrap_or("Default").to_string(),
//                 keyframes,
//                 timestamps,
//             })
//         }
//     }
//     Ok(animation_clips)
// }
