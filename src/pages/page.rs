use std::rc::Rc;

use ratatui::{crossterm::event::KeyEvent, layout::Rect, Frame};

use crate::{
    app::AppContext,
    event::Sender,
    object::{BucketItem, FileDetail, ObjectItem, ObjectKey, RawObject},
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
    pub fn handle_key(&mut self, key: KeyEvent) {
        match self {
            Page::Initializing(page) => page.handle_key(key),
            Page::BucketList(page) => page.handle_key(key),
            Page::ObjectList(page) => page.handle_key(key),
            Page::ObjectDetail(page) => page.handle_key(key),
            Page::ObjectPreview(page) => page.handle_key(key),
            Page::Help(page) => page.handle_key(key),
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        match self {
            Page::Initializing(page) => page.render(f, area),
            Page::BucketList(page) => page.render(f, area),
            Page::ObjectList(page) => page.render(f, area),
            Page::ObjectDetail(page) => page.render(f, area),
            Page::ObjectPreview(page) => page.render(f, area),
            Page::Help(page) => page.render(f, area),
        }
    }

    pub fn helps(&self) -> Vec<String> {
        match self {
            Page::Initializing(page) => page.helps(),
            Page::BucketList(page) => page.helps(),
            Page::ObjectList(page) => page.helps(),
            Page::ObjectDetail(page) => page.helps(),
            Page::ObjectPreview(page) => page.helps(),
            Page::Help(page) => page.helps(),
        }
    }

    pub fn short_helps(&self) -> Vec<(String, usize)> {
        match self {
            Page::Initializing(page) => page.short_helps(),
            Page::BucketList(page) => page.short_helps(),
            Page::ObjectList(page) => page.short_helps(),
            Page::ObjectDetail(page) => page.short_helps(),
            Page::ObjectPreview(page) => page.short_helps(),
            Page::Help(page) => page.short_helps(),
        }
    }
}

impl Page {
    pub fn of_initializing(ctx: Rc<AppContext>, tx: Sender) -> Self {
        Self::Initializing(Box::new(InitializingPage::new(ctx, tx)))
    }

    pub fn of_bucket_list(bucket_items: Vec<BucketItem>, ctx: Rc<AppContext>, tx: Sender) -> Self {
        Self::BucketList(Box::new(BucketListPage::new(bucket_items, ctx, tx)))
    }

    pub fn of_object_list(
        object_items: Vec<ObjectItem>,
        object_key: ObjectKey,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        Self::ObjectList(Box::new(ObjectListPage::new(
            object_items,
            object_key,
            ctx,
            tx,
        )))
    }

    pub fn of_object_detail(
        file_detail: FileDetail,
        object_items: Vec<ObjectItem>,
        object_key: ObjectKey,
        list_state: ScrollListState,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        Self::ObjectDetail(Box::new(ObjectDetailPage::new(
            file_detail,
            object_items,
            object_key,
            list_state,
            ctx,
            tx,
        )))
    }

    pub fn of_object_preview(
        file_detail: FileDetail,
        file_version_id: Option<String>,
        object: RawObject,
        object_key: ObjectKey,
        ctx: Rc<AppContext>,
        tx: Sender,
    ) -> Self {
        Self::ObjectPreview(Box::new(ObjectPreviewPage::new(
            file_detail,
            file_version_id,
            object,
            object_key,
            ctx,
            tx,
        )))
    }

    pub fn of_help(helps: Vec<String>, ctx: Rc<AppContext>, tx: Sender) -> Self {
        Self::Help(Box::new(HelpPage::new(helps, ctx, tx)))
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

    pub fn as_object_detail(&self) -> &ObjectDetailPage {
        match self {
            Self::ObjectDetail(page) => page,
            page => panic!("Page is not ObjectDetail: {:?}", page),
        }
    }

    pub fn as_mut_object_detail(&mut self) -> &mut ObjectDetailPage {
        match self {
            Self::ObjectDetail(page) => &mut *page,
            page => panic!("Page is not ObjectDetail: {:?}", page),
        }
    }

    pub fn as_object_preview(&self) -> &ObjectPreviewPage {
        match self {
            Self::ObjectPreview(page) => page,
            page => panic!("Page is not ObjectPreview: {:?}", page),
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
    pub fn new(ctx: Rc<AppContext>, tx: Sender) -> PageStack {
        PageStack {
            stack: vec![Page::of_initializing(ctx, tx)],
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

    pub fn iter(&self) -> std::slice::Iter<Page> {
        self.stack.iter()
    }
}
