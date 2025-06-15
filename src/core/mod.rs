use glam::Mat4;

pub mod app;
pub mod window;
pub mod renderer;
pub mod raytracer;
mod test;

/// Типы пайплайнов рендеринга
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineType {
    /// Простой пайплайн для отрисовки мешей
    Simple,
}

