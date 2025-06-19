use std::f32::consts::FRAC_PI_2;

use crate::math::angle::Angle;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuSkyState {
    params: [f32; 27],       // 0 byte offset, 108 byte size
    radiances: [f32; 3],     // 108 byte offset, 12 byte size
    _padding: [u32; 2],      // 120 byte offset, 8 byte size
    sun_direction: [f32; 4], // 128 byte offset, 16 byte size
}

#[derive(Clone, Copy, PartialEq)]
pub struct SkyParams {
    // Azimuth must be between 0..=360 degrees
    pub azimuth_degrees: f32,
    // Inclination must be between 0..=90 degrees
    pub zenith_degrees: f32,
    // Turbidity must be between 1..=10
    pub turbidity: f32,
    // Albedo elements must be between 0..=1
    pub albedo: [f32; 3],
}

impl Default for SkyParams {
    fn default() -> Self {
        Self {
            azimuth_degrees: 0_f32,
            zenith_degrees: 85_f32,
            turbidity: 4_f32,
            albedo: [1_f32; 3],
        }
    }
}

impl SkyParams {
    pub fn to_sky_state(self: &SkyParams) -> Result<GpuSkyState, hw_skymodel::rgb::Error> {
        let azimuth = Angle::degrees(self.azimuth_degrees).as_radians();
        let zenith = Angle::degrees(self.zenith_degrees).as_radians();
        let sun_direction = [
            zenith.sin() * azimuth.cos(),
            zenith.cos(),
            zenith.sin() * azimuth.sin(),
            0_f32,
        ];

        let state = hw_skymodel::rgb::SkyState::new(&hw_skymodel::rgb::SkyParams {
            elevation: FRAC_PI_2 - zenith,
            turbidity: self.turbidity,
            albedo: self.albedo,
        })?;

        let (params_data, radiance_data) = state.raw();

        Ok(GpuSkyState {
            params: params_data,
            radiances: radiance_data,
            _padding: [0_u32, 2],
            sun_direction,
        })
    }
}
