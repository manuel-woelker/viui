use crate::infrastructure::image_pool::ImagePool;
use crate::render::command::{ImageId, RenderCommand};
use crate::resource::Resource;
use crate::result::ViuiResult;

pub struct RenderContext<'a> {
    render_queue: Vec<RenderCommand>,
    image_pool: &'a mut ImagePool,
}

impl<'a> RenderContext<'a> {
    pub fn new(image_pool: &'a mut ImagePool) -> Self {
        Self {
            render_queue: Default::default(),
            image_pool,
        }
    }
}
impl RenderContext<'_> {
    pub fn add_command(&mut self, command: RenderCommand) {
        self.render_queue.push(command);
    }

    pub fn get_image_id(&mut self, path: &str) -> ViuiResult<ImageId> {
        if let Some(image_id) = self.image_pool.get(path) {
            return Ok(image_id);
        }
        let image_id = self.image_pool.new_image_id();
        self.add_command(RenderCommand::LoadImage {
            image_id,
            resource: Resource::new(path),
        });
        self.image_pool.set(path, image_id);
        Ok(image_id)
    }

    pub fn render_queue(self) -> Vec<RenderCommand> {
        self.render_queue
    }
}
