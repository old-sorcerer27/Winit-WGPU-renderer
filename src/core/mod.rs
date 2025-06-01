pub mod app;
pub mod window;
pub mod renderer;
mod test;

/// Типы пайплайнов рендеринга
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelineType {
    /// Простой пайплайн для отрисовки мешей
    Simple,
}

