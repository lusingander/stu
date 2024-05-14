use enum_tag::EnumTag;
use itsuki::zero_indexed_enum;
use std::sync::Arc;
use tokio::spawn;

use crate::{
    client::Client,
    component::{AppListState, AppListStates},
    config::Config,
    error::{AppError, Result},
    event::{
        AppEventType, AppKeyAction, AppKeyInput, CompleteDownloadObjectResult,
        CompleteInitializeResult, CompleteLoadObjectResult, CompleteLoadObjectsResult,
        CompletePreviewObjectResult, Sender,
    },
    file::{copy_to_clipboard, save_binary, save_error_log},
    keys::AppKeyActionManager,
    object::{AppObjects, BucketItem, FileDetail, FileVersion, Object, ObjectItem, ObjectKey},
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

zero_indexed_enum! {
    DetailViewState => [
        Detail,
        Version,
    ]
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone, Copy)]
pub struct CopyDetailViewState {
    pub selected: CopyDetailViewItemType,
    pub before: DetailViewState,
}

impl CopyDetailViewState {
    fn new(before: DetailViewState) -> CopyDetailViewState {
        CopyDetailViewState {
            selected: CopyDetailViewItemType::Key,
            before,
        }
    }
}

zero_indexed_enum! {
    CopyDetailViewItemType => [
        Key,
        S3Uri,
        Arn,
        ObjectUrl,
        Etag,
    ]
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

#[derive(Clone)] // fixme: object size can be large...
pub struct PreviewViewState {
    pub preview: Vec<String>,
    pub preview_len: usize,
    pub preview_max_digits: usize,
    pub offset: usize,
    path: String,
    obj: Object,
}

impl PreviewViewState {
    fn new(obj: Object, path: String) -> PreviewViewState {
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
    list_states: AppListStates,
    pub view_state: ViewState,
    pub notification: Notification,
    pub is_loading: bool,

    width: usize,
    height: usize,
}

impl AppViewState {
    fn new(width: usize, height: usize) -> AppViewState {
        AppViewState {
            list_states: AppListStates::new(list_area_height(height)),
            view_state: ViewState::Initializing,
            notification: Notification::None,
            is_loading: true,
            width,
            height,
        }
    }

    pub fn push_new_list_state(&mut self) {
        self.list_states.push_new();
    }

    pub fn pop_current_list_state(&mut self) {
        self.list_states.pop_current();
    }

    pub fn clear_list_state(&mut self) {
        self.list_states.clear();
    }

    pub fn current_list_state(&self) -> &AppListState {
        self.list_states.current()
    }

    pub fn current_list_state_mut(&mut self) -> &mut AppListState {
        self.list_states.current_mut()
    }

    pub fn reset_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.list_states.reset_height(list_area_height(height))
    }
}

pub struct App {
    pub action_manager: AppKeyActionManager,
    pub app_view_state: AppViewState,
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

    fn current_items_len(&self) -> usize {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::Detail(_)
            | ViewState::DetailSave(_)
            | ViewState::CopyDetail(_)
            | ViewState::Preview(_)
            | ViewState::PreviewSave(_)
            | ViewState::Help(_) => 0,
            ViewState::BucketList => self.bucket_items().len(),
            ViewState::ObjectList => self.current_object_items().len(),
        }
    }

    pub fn bucket_items(&self) -> Vec<BucketItem> {
        self.app_objects.get_bucket_items()
    }

    pub fn current_object_items(&self) -> Vec<ObjectItem> {
        self.app_objects
            .get_object_items(&self.current_object_key())
    }

    fn get_current_selected_bucket_item(&self) -> Option<&BucketItem> {
        let i = self.app_view_state.current_list_state().selected;
        self.app_objects.get_bucket_item(i)
    }

    fn get_current_selected_object_item(&self) -> Option<&ObjectItem> {
        let i = self.app_view_state.current_list_state().selected;
        self.app_objects
            .get_object_item(&self.current_object_key(), i)
    }

    pub fn get_current_file_detail(&self) -> Option<&FileDetail> {
        self.get_current_selected_object_item()
            .and_then(|selected| {
                if let ObjectItem::File { name, .. } = selected {
                    let key = &self.current_object_key_with_name(name.to_string());
                    self.app_objects.get_object_detail(key)
                } else {
                    None
                }
            })
    }

    pub fn get_current_file_versions(&self) -> Option<&Vec<FileVersion>> {
        self.get_current_selected_object_item()
            .and_then(|selected| {
                if let ObjectItem::File { name, .. } = selected {
                    let key = &self.current_object_key_with_name(name.to_string());
                    self.app_objects.get_object_versions(key)
                } else {
                    None
                }
            })
    }

    pub fn send_app_key_action(&self, action: AppKeyAction) {
        self.tx.send(AppEventType::KeyAction(action));
    }

    pub fn send_app_key_input(&self, input: AppKeyInput) {
        self.tx.send(AppEventType::KeyInput(input));
    }

    pub fn bucket_list_select_next(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            self.list_select_next();
        }
    }

    pub fn object_list_select_next(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            self.list_select_next();
        }
    }

    fn list_select_next(&mut self) {
        let current_selected = self.app_view_state.current_list_state().selected;
        let len = self.current_items_len();
        if len == 0 || current_selected >= len - 1 {
            self.app_view_state.current_list_state_mut().select_first();
        } else {
            self.app_view_state.current_list_state_mut().select_next();
        };
    }

    pub fn copy_detail_select_next(&mut self) {
        if let ViewState::CopyDetail(vs) = self.app_view_state.view_state {
            let vs = CopyDetailViewState {
                selected: vs.selected.next(),
                ..vs
            };
            self.app_view_state.view_state = ViewState::CopyDetail(vs);
        }
    }

    pub fn bucket_list_select_prev(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            self.list_select_prev();
        }
    }

    pub fn object_list_select_prev(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            self.list_select_prev();
        }
    }

    fn list_select_prev(&mut self) {
        let current_selected = self.app_view_state.current_list_state().selected;
        let len = self.current_items_len();
        if len == 0 {
            self.app_view_state.current_list_state_mut().select_first();
        } else if current_selected == 0 {
            self.app_view_state
                .current_list_state_mut()
                .select_last(len);
        } else {
            self.app_view_state.current_list_state_mut().select_prev();
        };
    }

    pub fn copy_detail_select_prev(&mut self) {
        if let ViewState::CopyDetail(vs) = self.app_view_state.view_state {
            let vs = CopyDetailViewState {
                selected: vs.selected.prev(),
                ..vs
            };
            self.app_view_state.view_state = ViewState::CopyDetail(vs);
        }
    }

    pub fn bucket_list_select_next_page(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            self.list_select_next_page();
        }
    }

    pub fn object_list_select_next_page(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            self.list_select_next_page();
        }
    }

    fn list_select_next_page(&mut self) {
        let len = self.current_items_len();
        self.app_view_state
            .current_list_state_mut()
            .select_next_page(len);
    }

    pub fn bucket_list_select_prev_page(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            self.list_select_prev_page();
        }
    }

    pub fn object_list_select_prev_page(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            self.list_select_prev_page();
        }
    }

    fn list_select_prev_page(&mut self) {
        let len = self.current_items_len();
        self.app_view_state
            .current_list_state_mut()
            .select_prev_page(len);
    }

    pub fn bucket_list_select_first(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            self.list_select_first();
        }
    }

    pub fn object_list_select_first(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            self.list_select_first();
        }
    }

    fn list_select_first(&mut self) {
        self.app_view_state.current_list_state_mut().select_first();
    }

    pub fn bucket_list_select_last(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            self.list_select_last();
        }
    }

    pub fn object_list_select_last(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            self.list_select_last();
        }
    }

    fn list_select_last(&mut self) {
        let len = self.current_items_len();
        self.app_view_state
            .current_list_state_mut()
            .select_last(len);
    }

    pub fn bucket_list_move_down(&mut self) {
        if let ViewState::BucketList = self.app_view_state.view_state {
            if let Some(selected) = self.get_current_selected_bucket_item() {
                self.current_bucket = Some(selected.to_owned());
                self.app_view_state.push_new_list_state();
                self.app_view_state.view_state = ViewState::ObjectList;

                if !self.exists_current_objects() {
                    self.tx.send(AppEventType::LoadObjects);
                    self.app_view_state.is_loading = true;
                }
            }
        }
    }

    pub fn object_list_move_down(&mut self) {
        if let ViewState::ObjectList = self.app_view_state.view_state {
            if let Some(selected) = self.get_current_selected_object_item() {
                if let ObjectItem::File { .. } = selected {
                    if self.exists_current_object_detail() {
                        self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);
                    } else {
                        self.tx.send(AppEventType::LoadObject);
                        self.app_view_state.is_loading = true;
                    }
                } else {
                    self.current_path.push(selected.name().to_owned());
                    self.app_view_state.push_new_list_state();

                    if !self.exists_current_objects() {
                        self.tx.send(AppEventType::LoadObjects);
                        self.app_view_state.is_loading = true;
                    }
                }
            }
        }
    }

    pub fn copy_detail_copy_selected_value(&self) {
        if let ViewState::CopyDetail(vs) = self.app_view_state.view_state {
            if let Some(detail) = self.get_current_file_detail() {
                let value = match vs.selected {
                    CopyDetailViewItemType::Key => &detail.key,
                    CopyDetailViewItemType::S3Uri => &detail.s3_uri,
                    CopyDetailViewItemType::Arn => &detail.arn,
                    CopyDetailViewItemType::ObjectUrl => &detail.object_url,
                    CopyDetailViewItemType::Etag => &detail.e_tag,
                };
                let (name, value) = (vs.selected.name().to_owned(), value.to_owned());
                self.tx.send(AppEventType::CopyToClipboard(name, value));
            }
        }
    }

    fn exists_current_object_detail(&self) -> bool {
        match self.get_current_selected_object_item() {
            Some(selected) => {
                let key = &self.current_object_key_with_name(selected.name().to_string());
                self.app_objects.exists_object_details(key)
            }
            None => false,
        }
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
            self.app_view_state.pop_current_list_state();
        }
    }

    pub fn detail_close(&mut self) {
        if let ViewState::Detail(_) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::ObjectList;
        }
    }

    pub fn copy_detail_close(&mut self) {
        if let ViewState::CopyDetail(vs) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::Detail(vs.before);
        }
    }

    pub fn preview_scroll_forward(&mut self) {
        if let ViewState::Preview(ref mut vs) = self.app_view_state.view_state {
            if vs.offset < vs.preview_len - 1 {
                vs.offset = vs.offset.saturating_add(1);
            }
        }
    }

    pub fn preview_scroll_backward(&mut self) {
        if let ViewState::Preview(ref mut vs) = self.app_view_state.view_state {
            if vs.offset > 0 {
                vs.offset = vs.offset.saturating_sub(1);
            }
        }
    }

    pub fn preview_scroll_to_top(&mut self) {
        if let ViewState::Preview(ref mut vs) = self.app_view_state.view_state {
            vs.offset = 0;
        }
    }

    pub fn preview_scroll_to_end(&mut self) {
        if let ViewState::Preview(ref mut vs) = self.app_view_state.view_state {
            vs.offset = vs.preview_len - 1;
        }
    }

    pub fn preview_close(&mut self) {
        if let ViewState::Preview(_) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);
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
            self.app_view_state.clear_list_state();
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
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn load_object(&self) {
        if let Some(ObjectItem::File {
            name, size_byte, ..
        }) = self.get_current_selected_object_item()
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
                    .set_object_details(map_key, *detail, versions);
                self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);
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
        }
    }

    pub fn toggle_help(&mut self) {
        match &self.app_view_state.view_state {
            ViewState::Initializing => {}
            ViewState::Help(before) => {
                self.app_view_state.view_state = *before.clone();
            }
            ViewState::BucketList
            | ViewState::ObjectList
            | ViewState::Detail(_)
            | ViewState::DetailSave(_)
            | ViewState::CopyDetail(_)
            | ViewState::Preview(_)
            | ViewState::PreviewSave(_) => {
                let before = self.app_view_state.view_state.clone();
                self.app_view_state.view_state = ViewState::Help(Box::new(before));
            }
        }
    }

    pub fn detail_download_object(&mut self) {
        if let ViewState::Detail(_) = self.app_view_state.view_state {
            self.tx.send(AppEventType::DownloadObject);
            self.app_view_state.is_loading = true;
        }
    }

    pub fn detail_open_download_object_as(&mut self) {
        if let ViewState::Detail(vs) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::DetailSave(DetailSaveViewState::new(vs))
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
                ViewState::PreviewSave(PreviewSaveViewState::new(*vs.clone()))
        }
    }

    pub fn detail_preview(&mut self) {
        if let ViewState::Detail(_) = self.app_view_state.view_state {
            self.tx.send(AppEventType::PreviewObject);
            self.app_view_state.is_loading = true;
        }
    }

    pub fn detail_open_copy_details(&mut self) {
        if let ViewState::Detail(vs) = self.app_view_state.view_state {
            self.app_view_state.view_state = ViewState::CopyDetail(CopyDetailViewState::new(vs));
        }
    }

    pub fn download_object(&self) {
        self.download_object_and(None, |tx, obj, path| {
            let result = CompleteDownloadObjectResult::new(obj, path);
            tx.send(AppEventType::CompleteDownloadObject(result));
        })
    }

    pub fn download_object_as(&self, input: String) {
        self.download_object_and(Some(&input), |tx, obj, path| {
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

    pub fn preview_object(&self) {
        self.download_object_and(None, |tx, obj, path| {
            let result = CompletePreviewObjectResult::new(obj, path);
            tx.send(AppEventType::CompletePreviewObject(result));
        })
    }

    pub fn complete_preview_object(&mut self, result: Result<CompletePreviewObjectResult>) {
        match result {
            Ok(CompletePreviewObjectResult { obj, path }) => {
                self.app_view_state.view_state =
                    ViewState::Preview(Box::new(PreviewViewState::new(obj, path)));
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        };
        self.clear_notification();
        self.app_view_state.is_loading = false;
    }

    fn download_object_and<F>(&self, save_file_name: Option<&str>, f: F)
    where
        F: Fn(Sender, Result<Object>, String) + Send + 'static,
    {
        if let Some(ObjectItem::File {
            name, size_byte, ..
        }) = self.get_current_selected_object_item()
        {
            let bucket = self.current_bucket();
            let prefix = self.current_object_prefix();
            let key = format!("{}{}", prefix, name);

            let config = self.config.as_ref().unwrap();
            let path = config.download_file_path(save_file_name.unwrap_or(name));

            let (client, tx) = self.unwrap_client_tx();
            let size_byte = *size_byte;
            let loading = self.handle_loading_size(size_byte, tx.clone());
            spawn(async move {
                let obj = client
                    .download_object(&bucket, &key, size_byte, loading)
                    .await;
                f(tx, obj, path);
            });
        }
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
            let (client, _) = self.unwrap_client_tx();
            let current_selected = self.get_current_selected_object_item();
            let result = if let Some(ObjectItem::File { name, .. }) = current_selected {
                let prefix = self.current_object_prefix();
                client.open_management_console_object(&self.current_bucket(), &prefix, name)
            } else {
                Err(AppError::msg("Failed to get current selected item"))
            };
            if let Err(e) = result {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
    }

    pub fn detail_save_download_object_as(&mut self) {
        if let ViewState::DetailSave(vs) = &self.app_view_state.view_state {
            let input = vs.input.trim().to_string();
            if !input.is_empty() {
                self.tx.send(AppEventType::DownloadObjectAs(input));
                self.app_view_state.is_loading = true;
            }
            self.app_view_state.view_state = ViewState::Detail(vs.before);
        }
    }

    pub fn preview_save_download_object_as(&mut self) {
        if let ViewState::PreviewSave(vs) = &self.app_view_state.view_state {
            let input = vs.input.trim().to_string();
            if !input.is_empty() {
                self.tx.send(AppEventType::DownloadObjectAs(input));
                self.app_view_state.is_loading = true;
            }
            self.app_view_state.view_state = ViewState::Preview(Box::new(vs.before.clone()));
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

    pub fn key_input(&mut self, input: AppKeyInput) {
        fn update(app_key_input: AppKeyInput, input: &mut String, cursor: &mut u16) {
            match app_key_input {
                AppKeyInput::Char(c) => {
                    if c == '?' {
                        return;
                    }
                    input.push(c);
                    *cursor = cursor.saturating_add(1);
                }
                AppKeyInput::Backspace => {
                    input.pop();
                    *cursor = cursor.saturating_sub(1);
                }
            }
        }
        match self.app_view_state.view_state {
            ViewState::DetailSave(ref mut vs) => update(input, &mut vs.input, &mut vs.cursor),
            ViewState::PreviewSave(ref mut vs) => update(input, &mut vs.input, &mut vs.cursor),
            _ => {}
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

fn list_area_height(height: usize) -> usize {
    height - 3 /* header */ - 2 /* footer */ - 2 /* list area border */
}
