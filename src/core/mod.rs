pub mod app;
pub mod window;
pub mod renderer;
mod test;

#[derive(Eq, PartialEq, Hash)]
enum PipelineType {
    Simple,
    Hard
}