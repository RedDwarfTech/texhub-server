use crate::model::app::app_context::AppContext;

pub struct TexProjectService<'a> {
    pub context: Option<&'a AppContext>,
}

impl<'a> Default for TexProjectService<'a> {
    fn default() -> Self {
        Self {
            context: None,
        }
    }
}