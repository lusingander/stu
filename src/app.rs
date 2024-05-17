use enum_tag::EnumTag;
use itsuki::zero_indexed_enum;
use std::sync::Arc;
use tokio::spawn;

use crate::{
    client::Client,
    config::Config,
    error::{AppError, Result},
    event::{
        AppEventType, AppKeyAction, AppKeyInput, CompleteDownloadObjectResult,
        CompleteInitializeResult, CompleteLoadObjectResult, CompleteLoadObjectsResult,
        CompletePreviewObjectResult, Sender,
    },
    file::{copy_to_clipboard, save_binary, save_error_log},
    keys::AppKeyActionManager,
    object::{AppObjects, BucketItem, FileDetail, Object, ObjectItem, ObjectKey},
    pages::page::Page,
    util::{digits, to_preview_string},
};

#[derive(Clone, EnumTag)]
pub enum ViewState {
    Initializing,
    BucketList,
    ObjectList,
    Detail(DetailViewState),
    DetailSave(DetailSaveViewState),
    CopyDetail(CopyDetailViewState),
    Preview(Box<PreviewViewState>),
    PreviewSave(PreviewSaveViewState),
    Help(Box<ViewState>),
}

pub type ViewStateTag = <ViewState as EnumTag>::Tag;

#[zero_indexed_enum]
pub enum DetailViewState {
    Detail,
    Version,
}

#[derive(Debug, Clone)]
pub struct DetailSaveViewState {
    pub input: String,
    pub cursor: u16,
    pub before: DetailViewState,
}

impl DetailSaveViewState {
    pub fn new(before: DetailViewState) -> DetailSaveViewState {
        DetailSaveViewState {
            input: String::new(),
            cursor: 0,
            before,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreviewSaveViewState {
    pub input: String,
    pub cursor: u16,
    pub before: PreviewViewState,
}

impl PreviewSaveViewState {
    pub fn new(before: PreviewViewState) -> PreviewSaveViewState {
        PreviewSaveViewState {
            input: String::new(),
            cursor: 0,
            before,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CopyDetailViewState {
    pub selected: CopyDetailViewItemType,
    pub before: DetailViewState,
}

impl CopyDetailViewState {
    pub fn new(before: DetailViewState) -> CopyDetailViewState {
        CopyDetailViewState {
            selected: CopyDetailViewItemType::Key,
            before,
        }
    }
}

#[derive(Default)]
#[zero_indexed_enum]
pub enum CopyDetailViewItemType {
    #[default]
    Key,
    S3Uri,
    Arn,
    ObjectUrl,
    Etag,
}

impl CopyDetailViewItemType {
    pub fn name(&self) -> &str {
        use CopyDetailViewItemType::*;
        match self {
            Key => "Key",
            S3Uri => "S3 URI",
            Arn => "ARN",
            ObjectUrl => "Object URL",
            Etag => "ETag",
        }
    }
}

#[derive(Debug, Clone)] // fixme: object size can be large...
pub struct PreviewViewState {
    pub preview: Vec<String>,
    pub preview_len: usize,
    pub preview_max_digits: usize,
    pub offset: usize,
    path: String,
    obj: Object,
}

impl PreviewViewState {
    pub fn new(obj: Object, path: String) -> PreviewViewState {
        let s = to_preview_string(&obj.bytes, &obj.content_type);
        let s = if s.ends_with('\n') {
            s.trim_end()
        } else {
            s.as_str()
        };
        let preview: Vec<String> = s.split('\n').map(|s| s.to_string()).collect();
        let preview_len = preview.len();
        let preview_max_digits = digits(preview_len);
        PreviewViewState {
            preview,
            preview_len,
            preview_max_digits,
            offset: 0,
            path,
            obj,
        }
    }
}

pub enum Notification {
    None,
    Info(String),
    Success(String),
    Error(String),
}

pub struct AppViewState {
    pub view_state: ViewState,
    pub notification: Notification,
    pub is_loading: bool,

    width: usize,
    height: usize,
}

impl AppViewState {
    fn new(width: usize, height: usize) -> AppViewState {
        AppViewState {
            view_state: ViewState::Initializing,
            notification: Notification::None,
            is_loading: true,
            width,
            height,
        }
    }

    pub fn reset_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
}

pub struct PageStack {
    stack: Vec<Page>,
}

impl PageStack {
    fn new() -> PageStack {
        PageStack {
            stack: vec![Page::of_initializing()],
        }
    }

    fn push(&mut self, page: Page) {
        self.stack.push(page);
    }

    fn pop(&mut self) -> Page {
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
}

pub struct App {
    pub action_manager: AppKeyActionManager,
    pub app_view_state: AppViewState,
    pub page_stack: PageStack,
    app_objects: AppObjects,
    current_bucket: Option<BucketItem>,
    current_path: Vec<String>,
    client: Option<Arc<Client>>,
    config: Option<Config>,
    tx: Sender,
}

impl App {
    pub fn new(tx: Sender, width: usize, height: usize) -> App {
        App {
            action_manager: AppKeyActionManager::new(),
            app_view_state: AppViewState::new(width, height),
            app_objects: AppObjects::new(),
            current_bucket: None,
            current_path: Vec::new(),
            page_stack: PageStack::new(),
            client: None,
            config: None,
            tx,
        }
    }

    pub fn initialize(&mut self, config: Config, client: Client, bucket: Option<String>) {
        self.config = Some(config);
        self.client = Some(Arc::new(client));

        let (client, tx) = self.unwrap_client_tx();
        spawn(async move {
            let buckets = match bucket {
                Some(name) => client.load_bucket(&name).await.map(|b| vec![b]),
                None => client.load_all_buckets().await,
            };
            let result = CompleteInitializeResult::new(buckets);
            tx.send(AppEventType::CompleteInitialize(result));
        });
    }

    pub fn complete_initialize(&mut self, result: Result<CompleteInitializeResult>) {
        match result {
            Ok(CompleteInitializeResult { buckets }) => {
                self.app_objects.set_bucket_items(buckets);
                self.app_view_state.view_state = ViewState::BucketList;

                let bucket_list_page = Page::of_bucket_list(self.bucket_items());
                self.page_stack.pop(); // remove initializing page
                self.page_stack.push(bucket_list_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }

        if self.bucket_items().len() == 1 {
            // bucket name is specified, or if there is only one bucket, open it.
            // since continues to load object, is_loading is not reset.
            self.bucket_list_move_down();
        } else {
            self.app_view_state.is_loading = false;
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.app_view_state.reset_size(width, height);
    }

    pub fn breadcrumb_strs(&self) -> Vec<String> {
        match &self.current_bucket {
            Some(b) => {
                let mut current_path = self.current_path.to_vec();
                current_path.insert(0, b.name.to_string());
                current_path
            }
            None => Vec::new(),
        }
    }

    fn current_bucket(&self) -> String {
        self.current_bucket.as_ref().unwrap().name.to_owned()
    }

    fn current_object_prefix(&self) -> String {
        let mut prefix = String::new();
        for key in &self.current_path {
            prefix.push_str(key);
            prefix.push('/');
        }
        prefix
    }

    fn current_object_key(&self) -> ObjectKey {
        ObjectKey {
            bucket_name: self.current_bucket(),
            object_path: self.current_path.to_vec(),
        }
    }

    fn current_object_key_with_name(&self, name: String) -> ObjectKey {
        let mut object_path = self.current_path.to_vec();
        object_path.push(name);
        ObjectKey {
            bucket_name: self.current_bucket(),
            object_path,
        }
    }

    pub fn bucket_items(&self) -> Vec<BucketItem> {
        self.app_objects.get_bucket_items()
    }

    pub fn current_object_items(&self) -> Vec<ObjectItem> {
        self.app_objects
            .get_object_items(&self.current_object_key())
    }

    pub fn send_app_key_action(&self, action: AppKeyAction) {
        self.tx.send(AppEventType::KeyAction(action));
    }

    pub fn send_app_key_input(&self, input: AppKeyInput) {
        self.tx.send(AppEventType::KeyInput(input));
    }

    pub fn bucket_list_select_next(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_bucket_list();
            page.select_next();
        }
    }

    pub fn object_list_select_next(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_list();
            page.select_next();
        }
    }

    pub fn copy_detail_select_next(&mut self) {
        if let ViewState::CopyDetail(vs) = self.app_view_state.view_state {
            let vs = CopyDetailViewState {
                selected: vs.selected.next(),
                ..vs
            };
            self.app_view_state.view_state = ViewState::CopyDetail(vs);

            let page = self.page_stack.current_page_mut().as_mut_object_detail();
            page.select_next_copy_detail_item();
        }
    }

    pub fn bucket_list_select_prev(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_bucket_list();
            page.select_prev();
        }
    }

    pub fn object_list_select_prev(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_list();
            page.select_prev();
        }
    }

    pub fn copy_detail_select_prev(&mut self) {
        if let ViewState::CopyDetail(vs) = self.app_view_state.view_state {
            let vs = CopyDetailViewState {
                selected: vs.selected.prev(),
                ..vs
            };
            self.app_view_state.view_state = ViewState::CopyDetail(vs);

            let page = self.page_stack.current_page_mut().as_mut_object_detail();
            page.select_prev_copy_detail_item();
        }
    }

    pub fn bucket_list_select_next_page(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_bucket_list();
            page.select_next_page();
        }
    }

    pub fn object_list_select_next_page(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_list();
            page.select_next_page();
        }
    }

    pub fn bucket_list_select_prev_page(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_bucket_list();
            page.select_prev_page();
        }
    }

    pub fn object_list_select_prev_page(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_list();
            page.select_prev_page();
        }
    }

    pub fn bucket_list_select_first(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_bucket_list();
            page.select_first();
        }
    }

    pub fn object_list_select_first(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_list();
            page.select_first();
        }
    }

    pub fn bucket_list_select_last(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_bucket_list();
            page.select_last();
        }
    }

    pub fn object_list_select_last(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_list();
            page.select_last();
        }
    }

    pub fn bucket_list_move_down(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            let bucket_page = self.page_stack.current_page().as_bucket_list();
            let selected = bucket_page.current_selected_item().to_owned();

            self.current_bucket = Some(selected);
            self.app_view_state.view_state = ViewState::ObjectList;

            if self.exists_current_objects() {
                let object_list_page = Page::of_object_list(self.current_object_items());
                self.page_stack.push(object_list_page);
            } else {
                self.tx.send(AppEventType::LoadObjects);
                self.app_view_state.is_loading = true;
            }
        }
    }

    pub fn object_list_move_down(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let object_page = self.page_stack.current_page().as_object_list();
            let selected = object_page.current_selected_item().to_owned();

            match selected {
                ObjectItem::File { name, .. } => {
                    if self.exists_current_object_detail(&name) {
                        self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);

                        let current_object_key =
                            &self.current_object_key_with_name(name.to_string());
                        let detail = self
                            .app_objects
                            .get_object_detail(current_object_key)
                            .unwrap();
                        let versions = self
                            .app_objects
                            .get_object_versions(current_object_key)
                            .unwrap();

                        let object_detail_page = Page::of_object_detail(
                            detail.clone(),
                            versions.clone(),
                            object_page.object_list().clone(),
                            object_page.list_state(),
                        );
                        self.page_stack.push(object_detail_page);
                    } else {
                        self.tx.send(AppEventType::LoadObject);
                        self.app_view_state.is_loading = true;
                    }
                }
                ObjectItem::Dir { .. } => {
                    self.current_path.push(selected.name().to_owned());

                    if self.exists_current_objects() {
                        let object_list_page = Page::of_object_list(self.current_object_items());
                        self.page_stack.push(object_list_page);
                    } else {
                        self.tx.send(AppEventType::LoadObjects);
                        self.app_view_state.is_loading = true;
                    }
                }
            }
        }
    }

    pub fn copy_detail_copy_selected_value(&self) {
        if let ViewState::CopyDetail(_) = self.app_view_state.view_state {
            let object_detail_page = self.page_stack.current_page().as_object_detail();

            if let Some((name, value)) = object_detail_page.copy_detail_dialog_selected() {
                self.tx.send(AppEventType::CopyToClipboard(name, value));
            }
        }
    }

    fn exists_current_object_detail(&self, object_name: &str) -> bool {
        let key = &self.current_object_key_with_name(object_name.to_string());
        self.app_objects.exists_object_details(key)
    }

    fn exists_current_objects(&self) -> bool {
        self.app_objects
            .exists_object_item(&self.current_object_key())
    }

    pub fn object_list_move_up(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let key = self.current_path.pop();
            if key.is_none() {
                if self.bucket_items().len() == 1 {
                    return;
                }
                self.app_view_state.view_state = ViewState::BucketList;
                self.current_bucket = None;
            }
            self.page_stack.pop();
        }
    }

    pub fn detail_close(&mut self) {
        if let ViewState::Detail(_) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::ObjectList;

            self.page_stack.pop(); // remove detail page
        }
    }

    pub fn copy_detail_close(&mut self) {
        if let ViewState::CopyDetail(vs) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::Detail(vs.before);

            let page = self.page_stack.current_page_mut().as_mut_object_detail();
            page.close_copy_detail_dialog();
        }
    }

    pub fn preview_scroll_forward(&mut self) {
        if let ViewState::Preview(_) = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_preview();
            page.scroll_forward();
        }
    }

    pub fn preview_scroll_backward(&mut self) {
        if let ViewState::Preview(_) = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_preview();
            page.scroll_backward();
        }
    }

    pub fn preview_scroll_to_top(&mut self) {
        if let ViewState::Preview(_) = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_preview();
            page.scroll_to_top();
        }
    }

    pub fn preview_scroll_to_end(&mut self) {
        if let ViewState::Preview(_) = self.app_view_state.view_state {
            let page = self.page_stack.current_page_mut().as_mut_object_preview();
            page.scroll_to_end();
        }
    }

    pub fn preview_close(&mut self) {
        if let ViewState::Preview(_) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);

            self.page_stack.pop(); // remove preview page
        }
    }

    pub fn help_close(&mut self) {
        if let ViewState::Help(_) = self.app_view_state.view_state {
            self.toggle_help();
        }
    }

    pub fn object_list_back_to_bucket_list(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            if self.bucket_items().len() == 1 {
                return;
            }
            self.app_view_state.view_state = ViewState::BucketList;
            self.current_bucket = None;
            self.current_path.clear();
            self.page_stack.clear();
        }
    }

    pub fn load_objects(&self) {
        let bucket = self.current_bucket();
        let prefix = self.current_object_prefix();
        let (client, tx) = self.unwrap_client_tx();
        spawn(async move {
            let items = client.load_objects(&bucket, &prefix).await;
            let result = CompleteLoadObjectsResult::new(items);
            tx.send(AppEventType::CompleteLoadObjects(result));
        });
    }

    pub fn complete_load_objects(&mut self, result: Result<CompleteLoadObjectsResult>) {
        match result {
            Ok(CompleteLoadObjectsResult { items }) => {
                self.app_objects
                    .set_object_items(self.current_object_key().to_owned(), items);

                let object_list_page = Page::of_object_list(self.current_object_items());
                self.page_stack.push(object_list_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn load_object(&self) {
        let object_page = self.page_stack.current_page().as_object_list();

        if let ObjectItem::File {
            name, size_byte, ..
        } = object_page.current_selected_item()
        {
            let name = name.clone();
            let size_byte = *size_byte;

            let bucket = self.current_bucket();
            let prefix = self.current_object_prefix();
            let key = format!("{}{}", prefix, name);

            let map_key = self.current_object_key_with_name(name.to_string());

            let (client, tx) = self.unwrap_client_tx();
            spawn(async move {
                let detail = client
                    .load_object_detail(&bucket, &key, &name, size_byte)
                    .await;
                let versions = client.load_object_versions(&bucket, &key).await;
                let result = CompleteLoadObjectResult::new(detail, versions, map_key);
                tx.send(AppEventType::CompleteLoadObject(result));
            });
        }
    }

    pub fn complete_load_object(&mut self, result: Result<CompleteLoadObjectResult>) {
        match result {
            Ok(CompleteLoadObjectResult {
                detail,
                versions,
                map_key,
            }) => {
                self.app_objects
                    .set_object_details(map_key, *detail.clone(), versions.clone());
                self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);

                let object_page = self.page_stack.current_page().as_object_list();

                let object_detail_page = Page::of_object_detail(
                    *detail.clone(),
                    versions.clone(),
                    object_page.object_list().clone(),
                    object_page.list_state(),
                );
                self.page_stack.push(object_detail_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn detail_select_tabs(&mut self) {
        if let ViewState::Detail(vs) = self.app_view_state.view_state {
            match vs {
                DetailViewState::Detail => {
                    self.app_view_state.view_state = ViewState::Detail(DetailViewState::Version);
                }
                DetailViewState::Version => {
                    self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);
                }
            }

            let page = self.page_stack.current_page_mut().as_mut_object_detail();
            page.toggle_tab();
        }
    }

    pub fn toggle_help(&mut self) {
        match &self.app_view_state.view_state {
            ViewState::Initializing => {}
            ViewState::Help(before) => {
                self.app_view_state.view_state = *before.clone();

                self.page_stack.pop(); // remove help page
            }
            ViewState::BucketList
            | ViewState::ObjectList
            | ViewState::Detail(_)
            | ViewState::DetailSave(_)
            | ViewState::CopyDetail(_)
            | ViewState::Preview(_)
            | ViewState::PreviewSave(_) => {
                let before = self.app_view_state.view_state.clone();
                self.app_view_state.view_state = ViewState::Help(Box::new(before.clone()));

                let helps = self.action_manager.helps(&before);
                let help_page = Page::of_help(helps.clone());
                self.page_stack.push(help_page);
            }
        }
    }

    pub fn detail_download_object(&mut self) {
        if let ViewState::Detail(_) = self.app_view_state.view_state {
            let object_detail_page = self.page_stack.current_page().as_object_detail();
            let file_detail = object_detail_page.file_detail();

            self.tx
                .send(AppEventType::DownloadObject(file_detail.clone()));
            self.app_view_state.is_loading = true;
        }
    }

    pub fn detail_open_download_object_as(&mut self) {
        if let ViewState::Detail(vs) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::DetailSave(DetailSaveViewState::new(vs));

            let page = self.page_stack.current_page_mut().as_mut_object_detail();
            page.open_save_dialog();
        }
    }

    pub fn preview_download_object(&self) {
        if let ViewState::Preview(vs) = &self.app_view_state.view_state {
            // object has been already downloaded, so send completion event to save file
            let result = CompleteDownloadObjectResult::new(Ok(vs.obj.clone()), vs.path.clone());
            self.tx.send(AppEventType::CompleteDownloadObject(result));
        }
    }

    pub fn preview_open_download_object_as(&mut self) {
        if let ViewState::Preview(vs) = &self.app_view_state.view_state {
            self.app_view_state.view_state =
                ViewState::PreviewSave(PreviewSaveViewState::new(*vs.clone()));

            let page = self.page_stack.current_page_mut().as_mut_object_preview();
            page.open_save_dialog();
        }
    }

    pub fn detail_preview(&mut self) {
        if let ViewState::Detail(_) = self.app_view_state.view_state {
            let object_detail_page = self.page_stack.current_page().as_object_detail();
            let file_detail = object_detail_page.file_detail();

            self.tx
                .send(AppEventType::PreviewObject(file_detail.clone()));
            self.app_view_state.is_loading = true;
        }
    }

    pub fn detail_open_copy_details(&mut self) {
        if let ViewState::Detail(vs) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::CopyDetail(CopyDetailViewState::new(vs));

            let page = self.page_stack.current_page_mut().as_mut_object_detail();
            page.open_copy_detail_dialog();
        }
    }

    pub fn download_object(&self, file_detail: FileDetail) {
        let object_name = file_detail.name;
        let size_byte = file_detail.size_byte;

        self.download_object_and(&object_name, size_byte, None, |tx, obj, path| {
            let result = CompleteDownloadObjectResult::new(obj, path);
            tx.send(AppEventType::CompleteDownloadObject(result));
        })
    }

    pub fn download_object_as(&self, file_detail: FileDetail, input: String) {
        let object_name = file_detail.name;
        let size_byte = file_detail.size_byte;

        self.download_object_and(&object_name, size_byte, Some(&input), |tx, obj, path| {
            let result = CompleteDownloadObjectResult::new(obj, path);
            tx.send(AppEventType::CompleteDownloadObject(result));
        })
    }

    pub fn complete_download_object(&mut self, result: Result<CompleteDownloadObjectResult>) {
        let result = match result {
            Ok(CompleteDownloadObjectResult { obj, path }) => {
                save_binary(&path, &obj.bytes).map(|_| path)
            }
            Err(e) => Err(e),
        };
        match result {
            Ok(path) => {
                let msg = format!("Download completed successfully: {}", path);
                self.tx.send(AppEventType::NotifySuccess(msg));
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn preview_object(&self, file_detail: FileDetail) {
        let object_name = file_detail.name.clone();
        let size_byte = file_detail.size_byte;

        self.download_object_and(&object_name, size_byte, None, |tx, obj, path| {
            let result = CompletePreviewObjectResult::new(obj, file_detail, path);
            tx.send(AppEventType::CompletePreviewObject(result));
        })
    }

    pub fn complete_preview_object(&mut self, result: Result<CompletePreviewObjectResult>) {
        match result {
            Ok(CompletePreviewObjectResult {
                obj,
                file_detail,
                path,
            }) => {
                let preview_view_state = PreviewViewState::new(obj, path);
                self.app_view_state.view_state =
                    ViewState::Preview(Box::new(preview_view_state.clone()));

                let object_preview_page = Page::of_object_preview(
                    file_detail,
                    preview_view_state.preview,
                    preview_view_state.preview_max_digits,
                );
                self.page_stack.push(object_preview_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        };
        self.clear_notification();
        self.app_view_state.is_loading = false;
    }

    fn download_object_and<F>(
        &self,
        object_name: &str,
        size_byte: usize,
        save_file_name: Option<&str>,
        f: F,
    ) where
        F: FnOnce(Sender, Result<Object>, String) + Send + 'static,
    {
        let bucket = self.current_bucket();
        let prefix = self.current_object_prefix();
        let key = format!("{}{}", prefix, object_name);

        let config = self.config.as_ref().unwrap();
        let path = config.download_file_path(save_file_name.unwrap_or(object_name));

        let (client, tx) = self.unwrap_client_tx();
        let loading = self.handle_loading_size(size_byte, tx.clone());
        spawn(async move {
            let obj = client
                .download_object(&bucket, &key, size_byte, loading)
                .await;
            f(tx, obj, path);
        });
    }

    fn handle_loading_size(&self, total_size: usize, tx: Sender) -> Box<dyn Fn(usize) + Send> {
        if total_size < 10_000_000 {
            return Box::new(|_| {});
        }
        let decimal_places = if total_size > 1_000_000_000 { 1 } else { 0 };
        let opt =
            humansize::FormatSizeOptions::from(humansize::DECIMAL).decimal_places(decimal_places);
        let total_s = humansize::format_size_i(total_size, opt);
        let f = move |current| {
            let percent = (current * 100) / total_size;
            let cur_s = humansize::format_size_i(current, opt);
            let msg = format!("{:3}% downloaded ({} out of {})", percent, cur_s, total_s);
            tx.send(AppEventType::NotifyInfo(msg));
        };
        Box::new(f)
    }

    pub fn bucket_list_open_management_console(&self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            let (client, _) = self.unwrap_client_tx();
            let result = client.open_management_console_buckets();
            if let Err(e) = result {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
    }

    pub fn object_list_open_management_console(&self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            let (client, _) = self.unwrap_client_tx();
            let bucket = &self.current_bucket();
            let prefix = self.current_object_prefix();
            let result = client.open_management_console_list(bucket, &prefix);
            if let Err(e) = result {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
    }

    pub fn detail_open_management_console(&self) {
        if let ViewState::Detail(_) = self.app_view_state.view_state {
            let object_detail_page = self.page_stack.current_page().as_object_detail();

            let (client, _) = self.unwrap_client_tx();
            let prefix = self.current_object_prefix();

            let result = client.open_management_console_object(
                &self.current_bucket(),
                &prefix,
                &object_detail_page.file_detail().name,
            );
            if let Err(e) = result {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
    }

    pub fn detail_save_download_object_as(&mut self) {
        if let ViewState::DetailSave(vs) = &self.app_view_state.view_state {
            let object_detail_page = self.page_stack.current_page().as_object_detail();
            let file_detail = object_detail_page.file_detail();

            if let Some(input) = object_detail_page.save_dialog_key_input() {
                let input = input.trim().to_string();
                if !input.is_empty() {
                    self.tx
                        .send(AppEventType::DownloadObjectAs(file_detail.clone(), input));
                    self.app_view_state.is_loading = true;
                }
                self.app_view_state.view_state = ViewState::Detail(vs.before);

                let page = self.page_stack.current_page_mut().as_mut_object_detail();
                page.close_save_dialog();
            }
        }
    }

    pub fn preview_save_download_object_as(&mut self) {
        if let ViewState::PreviewSave(vs) = &self.app_view_state.view_state {
            let object_preview_page = self.page_stack.current_page().as_object_preview();
            let file_detail = object_preview_page.file_detail();

            if let Some(input) = object_preview_page.save_dialog_key_input() {
                let input = input.trim().to_string();
                if !input.is_empty() {
                    self.tx
                        .send(AppEventType::DownloadObjectAs(file_detail.clone(), input));
                    self.app_view_state.is_loading = true;
                }
                self.app_view_state.view_state = ViewState::Preview(Box::new(vs.before.clone()));

                let page = self.page_stack.current_page_mut().as_mut_object_preview();
                page.close_save_dialog();
            }
        }
    }

    pub fn copy_to_clipboard(&self, name: String, value: String) {
        match copy_to_clipboard(value) {
            Ok(_) => {
                let msg = format!("Copied '{}' to clipboard successfully", name);
                self.tx.send(AppEventType::NotifySuccess(msg));
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
    }

    pub fn clear_notification(&mut self) {
        self.app_view_state.notification = Notification::None;
    }

    pub fn info_notification(&mut self, msg: String) {
        self.app_view_state.notification = Notification::Info(msg);
    }

    pub fn success_notification(&mut self, msg: String) {
        self.app_view_state.notification = Notification::Success(msg);
    }

    pub fn error_notification(&mut self, e: AppError) {
        self.save_error(&e);
        self.app_view_state.notification = Notification::Error(e.msg);
    }

    fn save_error(&self, e: &AppError) {
        let config = self.config.as_ref().unwrap();
        // cause panic if save errors
        let path = config.error_log_path().unwrap();
        save_error_log(&path, e).unwrap();
    }

    fn unwrap_client_tx(&self) -> (Arc<Client>, Sender) {
        (self.client.as_ref().unwrap().clone(), self.tx.clone())
    }
}
