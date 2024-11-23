use crate::infrastructure::font_pool::{FontIndex, FontPool};
use crate::infrastructure::image_pool::ImagePool;
use crate::infrastructure::measure_text::TextMeasurement;
use crate::render::command::{ImageId, RenderCommand};
use crate::resource::Resource;
use crate::result::ViuiResult;
use crate::types::Float;

pub struct RenderContext<'a> {
    font_size: Float,
    render_queue: Vec<RenderCommand>,
    image_pool: &'a mut ImagePool,
    font_pool: &'a mut FontPool,
    time: Float,
    is_animated: bool,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        image_pool: &'a mut ImagePool,
        font_pool: &'a mut FontPool,
        time: Float,
    ) -> ViuiResult<Self> {
        let mut render_queue = vec![];
        // Add images to render queue
        // TODO: handle multiple backends
        for image_path in std::mem::take(&mut image_pool.images_to_load) {
            let image_id = image_pool.get_image_id(&image_path)?;
            render_queue.push(RenderCommand::LoadImage {
                image_id,
                resource: Resource::from_path(image_path),
            });
        }

        Ok(Self {
            font_size: 25.0,
            render_queue,
            image_pool,
            font_pool,
            time,
            is_animated: false,
        })
    }
}
impl RenderContext<'_> {
    pub fn add_command(&mut self, command: RenderCommand) {
        self.render_queue.push(command);
    }
    pub fn add_commands(&mut self, commands: impl IntoIterator<Item = RenderCommand>) {
        self.render_queue.extend(commands);
    }

    pub fn get_image_id(&mut self, path: &str) -> ViuiResult<ImageId> {
        self.image_pool.get_image_id(path)
    }

    pub fn render_queue(self) -> Vec<RenderCommand> {
        self.render_queue
    }

    pub fn time(&self) -> Float {
        self.time
    }

    pub fn set_animated(&mut self) {
        self.is_animated = true;
    }

    pub fn reset_animated(&mut self) -> bool {
        let was_animated = self.is_animated;
        self.is_animated = false;
        was_animated
    }

    pub fn measure_text(&self, text: &str) -> ViuiResult<TextMeasurement> {
        self.font_pool
            .measure_text(FontIndex::new(0), text, self.font_size)
    }
}
