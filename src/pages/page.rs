use ratatui::{layout::Rect, Frame};

use crate::pages::{
    bucket_list::BucketListPage, help::HelpPage, initializing::InitializingPage,
    object_detail::ObjectDetailPage, object_list::ObjectListPage,
    object_preview::ObjectPreviewPage,
};

pub enum Page {
    Initializing(Box<InitializingPage>),
    BucketList(Box<BucketListPage>),
    ObjectList(Box<ObjectListPage>),
    ObjectDetail(Box<ObjectDetailPage>),
    ObjectPreview(Box<ObjectPreviewPage>),
    Help(Box<HelpPage>),
}

impl Page {
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self {
            Self::Initializing(page) => page.render(f, area),
            Self::BucketList(page) => page.render(f, area),
            Self::ObjectList(page) => page.render(f, area),
            Self::ObjectDetail(page) => page.render(f, area),
            Self::ObjectPreview(page) => page.render(f, area),
            Self::Help(page) => page.render(f, area),
        }
    }
}
