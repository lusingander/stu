use std::sync::{
    mpsc::{self, Sender},
    Arc,
};
use tokio::spawn;

use crate::{
    client::Client,
    component::{AppListState, AppListStates},
    config::Config,
    error::{AppError, Result},
    event::{
        AppEventType, CompleteDownloadObjectResult, CompleteInitializeResult,
        CompleteLoadObjectResult, CompleteLoadObjectsResult, CompletePreviewObjectResult,
    },
    file::{save_binary, save_error_log},
    item::{AppObjects, BucketItem, FileDetail, FileVersion, Object, ObjectItem, ObjectKey},
    util,
};

#[derive(Clone)]
pub enum ViewState {
    Initializing,
    BucketList,
    ObjectList,
    Detail(DetailViewState),
    Preview(Box<PreviewViewState>),
    Help(Box<ViewState>),
}

#[derive(Clone, Copy)]
pub enum DetailViewState {
    Detail = 0,
    Version = 1,
}

#[derive(Clone)]
pub struct PreviewViewState {
    pub preview: String,
    path: String,
    obj: Object,
}

pub enum Notification {
    None,
    Info(String),
    Error(String),
}

pub struct AppViewState {
    list_states: AppListStates,
    pub view_state: ViewState,
    pub notification: Notification,
    pub is_loading: bool,
}

impl AppViewState {
    fn new(height: usize) -> AppViewState {
        AppViewState {
            list_states: AppListStates::new(list_area_height(height)),
            view_state: ViewState::Initializing,
            notification: Notification::None,
            is_loading: true,
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

    pub fn reset_height(&mut self, height: usize) {
        self.list_states.reset_height(height)
    }
}

pub struct App {
    pub app_view_state: AppViewState,
    app_objects: AppObjects,
    current_bucket: Option<BucketItem>,
    current_path: Vec<String>,
    client: Option<Arc<Client>>,
    config: Option<Config>,
    tx: mpsc::Sender<AppEventType>,
}

impl App {
    pub fn new(tx: mpsc::Sender<AppEventType>, height: usize) -> App {
        App {
            app_view_state: AppViewState::new(height),
            app_objects: AppObjects::new(),
            current_bucket: None,
            current_path: Vec::new(),
            client: None,
            config: None,
            tx,
        }
    }

    pub fn initialize(&mut self, config: Config, client: Client) {
        self.config = Some(config);
        self.client = Some(Arc::new(client));

        let (client, tx) = self.unwrap_client_tx();
        spawn(async move {
            let buckets = client.load_all_buckets().await;
            let result = CompleteInitializeResult::new(buckets);
            tx.send(AppEventType::CompleteInitialize(result)).unwrap();
        });
    }

    pub fn complete_initialize(&mut self, result: Result<CompleteInitializeResult>) {
        match result {
            Ok(CompleteInitializeResult { buckets }) => {
                self.app_objects.set_bucket_items(buckets);
                self.app_view_state.view_state = ViewState::BucketList;
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn resize(&mut self, height: usize) {
        let h = list_area_height(height);
        self.app_view_state.reset_height(h);
        // todo: adjust
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
            | ViewState::Preview(_)
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

    pub fn select_next(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::Detail(_)
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::BucketList | ViewState::ObjectList => {
                let current_selected = self.app_view_state.current_list_state().selected;
                let len = self.current_items_len();
                if len == 0 || current_selected >= len - 1 {
                    self.app_view_state.current_list_state_mut().select_first();
                } else {
                    self.app_view_state.current_list_state_mut().select_next();
                };
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::Detail(_)
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::BucketList | ViewState::ObjectList => {
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
        }
    }

    pub fn select_next_page(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::Detail(_)
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::BucketList | ViewState::ObjectList => {
                let len = self.current_items_len();
                self.app_view_state
                    .current_list_state_mut()
                    .select_next_page(len)
            }
        }
    }

    pub fn select_prev_page(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::Detail(_)
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::BucketList | ViewState::ObjectList => {
                let len = self.current_items_len();
                self.app_view_state
                    .current_list_state_mut()
                    .select_prev_page(len)
            }
        }
    }

    pub fn select_first(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::Detail(_)
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::BucketList | ViewState::ObjectList => {
                self.app_view_state.current_list_state_mut().select_first();
            }
        }
    }

    pub fn select_last(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::Detail(_)
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::BucketList | ViewState::ObjectList => {
                let len = self.current_items_len();
                self.app_view_state
                    .current_list_state_mut()
                    .select_last(len);
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::Detail(_)
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::BucketList => {
                if let Some(selected) = self.get_current_selected_bucket_item() {
                    self.current_bucket = Some(selected.to_owned());
                    self.app_view_state.push_new_list_state();
                    self.app_view_state.view_state = ViewState::ObjectList;

                    if !self.exists_current_objects() {
                        self.tx.send(AppEventType::LoadObjects).unwrap();
                        self.app_view_state.is_loading = true;
                    }
                }
            }
            ViewState::ObjectList => {
                if let Some(selected) = self.get_current_selected_object_item() {
                    if let ObjectItem::File { .. } = selected {
                        if self.exists_current_object_detail() {
                            self.app_view_state.view_state =
                                ViewState::Detail(DetailViewState::Detail);
                        } else {
                            self.tx.send(AppEventType::LoadObject).unwrap();
                            self.app_view_state.is_loading = true;
                        }
                    } else {
                        self.current_path.push(selected.name().to_owned());
                        self.app_view_state.push_new_list_state();

                        if !self.exists_current_objects() {
                            self.tx.send(AppEventType::LoadObjects).unwrap();
                            self.app_view_state.is_loading = true;
                        }
                    }
                }
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

    pub fn move_up(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::BucketList => {}
            ViewState::ObjectList => {
                let key = self.current_path.pop();
                if key.is_none() {
                    self.app_view_state.view_state = ViewState::BucketList;
                    self.current_bucket = None;
                }
                self.app_view_state.pop_current_list_state();
            }
            ViewState::Detail(_) => {
                self.app_view_state.view_state = ViewState::ObjectList;
            }
            ViewState::Preview(_) => {
                self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);
            }
            ViewState::Help(_) => {
                self.toggle_help();
            }
        }
    }

    pub fn back_to_bucket_list(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::BucketList
            | ViewState::Detail(_)
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::ObjectList => {
                self.app_view_state.view_state = ViewState::BucketList;
                self.current_bucket = None;
                self.current_path.clear();
                self.app_view_state.clear_list_state();
            }
        }
    }

    pub fn load_objects(&self) {
        let bucket = self.current_bucket();
        let prefix = self.current_object_prefix();
        let (client, tx) = self.unwrap_client_tx();
        spawn(async move {
            let items = client.load_objects(&bucket, &prefix).await;
            let result = CompleteLoadObjectsResult::new(items);
            tx.send(AppEventType::CompleteLoadObjects(result)).unwrap();
        });
    }

    pub fn complete_load_objects(&mut self, result: Result<CompleteLoadObjectsResult>) {
        match result {
            Ok(CompleteLoadObjectsResult { items }) => {
                self.app_objects
                    .set_object_items(self.current_object_key().to_owned(), items);
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
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
                tx.send(AppEventType::CompleteLoadObject(result)).unwrap();
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
                self.tx.send(AppEventType::Error(e)).unwrap();
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn select_tabs(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::BucketList
            | ViewState::ObjectList
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::Detail(vs) => match vs {
                DetailViewState::Detail => {
                    self.app_view_state.view_state = ViewState::Detail(DetailViewState::Version);
                }
                DetailViewState::Version => {
                    self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);
                }
            },
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
            | ViewState::Preview(_) => {
                let before = self.app_view_state.view_state.clone();
                self.app_view_state.view_state = ViewState::Help(Box::new(before));
            }
        }
    }

    pub fn download(&mut self) {
        match &self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::BucketList
            | ViewState::ObjectList
            | ViewState::Help(_) => {}
            ViewState::Detail(_) => {
                self.tx.send(AppEventType::DownloadObject).unwrap();
                self.app_view_state.is_loading = true;
            }
            ViewState::Preview(vs) => {
                // object has been already downloaded, so send completion event to save file
                let result = CompleteDownloadObjectResult::new(Ok(vs.obj.clone()), vs.path.clone());
                self.tx
                    .send(AppEventType::CompleteDownloadObject(result))
                    .unwrap();
            }
        }
    }

    pub fn preview(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing
            | ViewState::BucketList
            | ViewState::ObjectList
            | ViewState::Preview(_)
            | ViewState::Help(_) => {}
            ViewState::Detail(_) => {
                self.tx.send(AppEventType::PreviewObject).unwrap();
                self.app_view_state.is_loading = true;
            }
        }
    }

    pub fn download_object(&self) {
        self.download_object_and(|tx, obj, path| {
            let result = CompleteDownloadObjectResult::new(obj, path);
            tx.send(AppEventType::CompleteDownloadObject(result))
                .unwrap();
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
                self.tx.send(AppEventType::Info(msg)).unwrap();
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn preview_object(&self) {
        self.download_object_and(|tx, obj, path| {
            let result = CompletePreviewObjectResult::new(obj, path);
            tx.send(AppEventType::CompletePreviewObject(result))
                .unwrap();
        })
    }

    pub fn complete_preview_object(&mut self, result: Result<CompletePreviewObjectResult>) {
        match result {
            Ok(CompletePreviewObjectResult { obj, path }) => {
                let preview = util::to_preview_string(&obj.bytes, &obj.content_type);
                let state = PreviewViewState { preview, path, obj };
                self.app_view_state.view_state = ViewState::Preview(Box::new(state));
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
            }
        };
        self.app_view_state.is_loading = false;
    }

    fn download_object_and<F>(&self, f: F)
    where
        F: Fn(Sender<AppEventType>, Result<Object>, String) + Send + 'static,
    {
        if let Some(ObjectItem::File { name, .. }) = self.get_current_selected_object_item() {
            let bucket = self.current_bucket();
            let prefix = self.current_object_prefix();
            let key = format!("{}{}", prefix, name);

            let config = self.config.as_ref().unwrap();
            let path = config.download_file_path(name);

            let (client, tx) = self.unwrap_client_tx();
            spawn(async move {
                let obj = client.download_object(&bucket, &key).await;
                f(tx, obj, path);
            });
        }
    }

    pub fn save_error(&self, e: &AppError) {
        let config = self.config.as_ref().unwrap();
        // cause panic if save errors
        let path = config.error_log_path().unwrap();
        save_error_log(&path, e).unwrap();
    }

    pub fn open_management_console(&self) {
        let (client, _) = self.unwrap_client_tx();

        let result = match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Preview(_) | ViewState::Help(_) => Ok(()),
            ViewState::BucketList => client.open_management_console_buckets(),
            ViewState::ObjectList => {
                let bucket = &self.current_bucket();
                let prefix = self.current_object_prefix();
                client.open_management_console_list(bucket, &prefix)
            }
            ViewState::Detail(_) => {
                if let Some(ObjectItem::File { name, .. }) = self.get_current_selected_object_item()
                {
                    let prefix = self.current_object_prefix();
                    client.open_management_console_object(&self.current_bucket(), &prefix, name)
                } else {
                    Err(AppError::msg("Failed to get current selected item"))
                }
            }
        };
        if let Err(e) = result {
            self.tx.send(AppEventType::Error(e)).unwrap();
        }
    }

    fn unwrap_client_tx(&self) -> (Arc<Client>, mpsc::Sender<AppEventType>) {
        (self.client.as_ref().unwrap().clone(), self.tx.clone())
    }
}

fn list_area_height(height: usize) -> usize {
    height - 3 /* header */ - 2 /* footer */ - 2 /* list area border */
}
