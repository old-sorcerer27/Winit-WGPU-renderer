use super::{buffer::BufferData, storage::Storage, Handle, Resource};
use std::path::Path;

use gltf::Gltf;

#[derive(Debug, Clone)] 
pub struct Animation {
}

impl Resource for Animation {
    type Key = super::TextureKey;
    
    type LoadParams = Animation;
    
    fn load(params: Self::LoadParams) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized {
        todo!()
    }
}

pub type AnimationStorage = Storage<Animation>;
pub type AnimationHandle = Handle<Animation>;

pub fn load_gltf_animations(
    gltf: &Gltf,
    base_path: Option<&Path>,
    buffers: &[BufferData],  
){
    for animation in gltf.animations() {
        for channel in animation.channels() {
            let reader = channel.reader(|buffer| Some(&buffers[buffer.index()].data));
            let keyframe_timestamps = if let Some(inputs) = reader.read_inputs() {
                match inputs {
                    gltf::accessor::Iter::Standard(times) => {
                        let times: Vec<f32> = times.collect();
                        println!("Time: {}", times.len());
                        // dbg!(times);
                    }
                    gltf::accessor::Iter::Sparse(_) => {
                        println!("Sparse keyframes not supported");
                    }
                }
            };

            let mut keyframes_vec: Vec<Vec<f32>> = Vec::new();
            let keyframes = if let Some(outputs) = reader.read_outputs() {
                match outputs {
                    gltf::animation::util::ReadOutputs::Translations(translation) => {
                        translation.for_each(|tr| {
                            // println!("Translation:");
                            // dbg!(tr);
                            let vector: Vec<f32> = tr.into();
                            keyframes_vec.push(vector);
                        });
                    },
                    other => ()
                    // gltf::animation::util::ReadOutputs::Rotations(_) => todo!(),
                    // gltf::animation::util::ReadOutputs::Scales(_) => todo!(),
                    // gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => todo!(),
                }
            };

            println!("Keyframes: {}", keyframes_vec.len());
        }
    }
}
