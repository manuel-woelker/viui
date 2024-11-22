use crate::infrastructure::styling::Styling;
use crate::result::ViuiResult;

pub struct RenderParameters<'a> {
    styling: &'a Styling,
}

impl<'a> RenderParameters<'a> {
    pub fn new(styling: &'a Styling) -> ViuiResult<Self> {
        Ok(Self { styling })
    }
}
impl RenderParameters<'_> {
    pub fn styling(&self) -> &Styling {
        self.styling
    }
}
