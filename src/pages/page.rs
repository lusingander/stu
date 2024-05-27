use crate::{
    config::PreviewConfig,
    event::Sender,
    object::{BucketItem, FileDetail, FileVersion, ObjectItem, RawObject},
    pages::{
        bucket_list::BucketListPage, help::HelpPage, initializing::InitializingPage,
        object_detail::ObjectDetailPage, object_list::ObjectListPage,
        object_preview::ObjectPreviewPage,
    },
    widget::ScrollListState,
};

#[derive(Debug)]
pub enum Page {
    Initializing(Box<InitializingPage>),
    BucketList(Box<BucketListPage>),
    ObjectList(Box<ObjectListPage>),
    ObjectDetail(Box<ObjectDetailPage>),
    ObjectPreview(Box<ObjectPreviewPage>),
    Help(Box<HelpPage>),
}

impl Page {
    pub fn of_initializing(tx: Sender) -> Self {
        Self::Initializing(Box::new(InitializingPage::new(tx)))
    }

    pub fn of_bucket_list(bucket_items: Vec<BucketItem>, tx: Sender) -> Self {
        Self::BucketList(Box::new(BucketListPage::new(bucket_items, tx)))
    }

    pub fn of_object_list(object_items: Vec<ObjectItem>, tx: Sender) -> Self {
        Self::ObjectList(Box::new(ObjectListPage::new(object_items, tx)))
    }

    pub fn of_object_detail(
        file_detail: FileDetail,
        file_versions: Vec<FileVersion>,
        object_items: Vec<ObjectItem>,
        list_state: ScrollListState,
        tx: Sender,
    ) -> Self {
        Self::ObjectDetail(Box::new(ObjectDetailPage::new(
            file_detail,
            file_versions,
            object_items,
            list_state,
            tx,
        )))
    }

    pub fn of_object_preview(
        file_detail: FileDetail,
        file_version_id: Option<String>,
        object: RawObject,
        path: String,
        preview_config: PreviewConfig,
        tx: Sender,
    ) -> Self {
        Self::ObjectPreview(Box::new(ObjectPreviewPage::new(
            file_detail,
            file_version_id,
            object,
            path,
            preview_config,
            tx,
        )))
    }

    pub fn of_help(helps: Vec<String>, tx: Sender) -> Self {
        Self::Help(Box::new(HelpPage::new(helps, tx)))
    }

    pub fn as_bucket_list(&self) -> &BucketListPage {
        match self {
            Self::BucketList(page) => page,
            page => panic!("Page is not BucketList: {:?}", page),
        }
    }

    pub fn as_object_list(&self) -> &ObjectListPage {
        match self {
            Self::ObjectList(page) => page,
            page => panic!("Page is not ObjectList: {:?}", page),
        }
    }

    pub fn as_mut_object_detail(&mut self) -> &mut ObjectDetailPage {
        match self {
            Self::ObjectDetail(page) => &mut *page,
            page => panic!("Page is not ObjectDetail: {:?}", page),
        }
    }

    pub fn as_mut_object_preview(&mut self) -> &mut ObjectPreviewPage {
        match self {
            Self::ObjectPreview(page) => &mut *page,
            page => panic!("Page is not ObjectPreview: {:?}", page),
        }
    }
}

#[derive(Debug)]
pub struct PageStack {
    stack: Vec<Page>,
}

impl PageStack {
    pub fn new(tx: Sender) -> PageStack {
        PageStack {
            stack: vec![Page::of_initializing(tx)],
        }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, page: Page) {
        self.stack.push(page);
    }

    pub fn pop(&mut self) -> Page {
        self.stack.pop().unwrap()
    }

    pub fn clear(&mut self) {
        self.stack.truncate(1);
    }

    pub fn current_page(&self) -> &Page {
        self.stack.last().unwrap()
    }

    pub fn current_page_mut(&mut self) -> &mut Page {
        self.stack.last_mut().unwrap()
    }

    pub fn head(&self) -> &Page {
        self.stack.first().unwrap()
    }

    pub fn iter(&self) -> std::slice::Iter<Page> {
        self.stack.iter()
    }
}
