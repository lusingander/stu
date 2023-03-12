use chrono::{DateTime, Local};
use std::{collections::HashMap, error::Error, sync::mpsc};
use tui::widgets::ListState;

use crate::{client::Client, config::Config, event::AppEventType, file::save_binary};

pub struct App {
    pub app_view_state: AppViewState,
    app_objects: AppObjects,
    current_keys: Vec<String>,
    client: Option<Client>,
    config: Option<Config>,
    tx: mpsc::Sender<AppEventType>,
}

pub struct AppViewState {
    pub current_list_state: ListState,
    pub view_state: ViewState,
    pub before_view_state: Option<ViewState>,
    pub notification: Notification,
    pub is_loading: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ViewState {
    Initializing,
    Default,
    ObjectDetail(FileDetailViewState),
    Help,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FileDetailViewState {
    Detail = 0,
    Version = 1,
}

pub enum Notification {
    None,
    Info(String),
    Error(String),
}

impl AppViewState {
    fn new() -> AppViewState {
        let mut current_list_state = ListState::default();
        current_list_state.select(Some(0));

        AppViewState {
            current_list_state,
            view_state: ViewState::Initializing,
            notification: Notification::None,
            before_view_state: None,
            is_loading: true,
        }
    }
}

struct AppObjects {
    items_map: HashMap<Vec<String>, Vec<Item>>,
    detail_map: HashMap<String, FileDetail>,
    versions_map: HashMap<String, Vec<FileVersion>>,
}

impl AppObjects {
    fn new() -> AppObjects {
        AppObjects {
            items_map: HashMap::new(),
            detail_map: HashMap::new(),
            versions_map: HashMap::new(),
        }
    }

    fn get_items(&self, keys: &[String]) -> Vec<Item> {
        self.items_map.get(keys).unwrap_or(&Vec::new()).to_vec()
    }

    fn get_items_len(&self, keys: &[String]) -> usize {
        self.items_map.get(keys).unwrap_or(&Vec::new()).len()
    }

    fn get_item(&self, keys: &[String], idx: usize) -> Option<&Item> {
        self.items_map.get(keys).and_then(|items| items.get(idx))
    }

    fn set_items(&mut self, keys: Vec<String>, items: Vec<Item>) {
        self.items_map.insert(keys, items);
    }

    fn exists_item(&self, keys: &[String]) -> bool {
        self.items_map.contains_key(keys)
    }

    fn get_object_detail(&self, key: &str) -> Option<&FileDetail> {
        self.detail_map.get(key)
    }

    fn get_object_versions(&self, key: &str) -> Option<&Vec<FileVersion>> {
        self.versions_map.get(key)
    }

    fn set_object_details(&mut self, key: &str, detail: FileDetail, versions: Vec<FileVersion>) {
        self.detail_map.insert(key.to_string(), detail);
        self.versions_map.insert(key.to_string(), versions);
    }

    fn exists_object_details(&self, key: &str) -> bool {
        self.detail_map.contains_key(key) && self.versions_map.contains_key(key)
    }
}

impl App {
    pub fn new(tx: mpsc::Sender<AppEventType>) -> App {
        App {
            app_view_state: AppViewState::new(),
            app_objects: AppObjects::new(),
            current_keys: Vec::new(),
            client: None,
            config: None,
            tx,
        }
    }

    pub async fn initialize(&mut self, config: Config, client: Client) {
        self.config = Some(config);
        self.client = Some(client);

        let client = self.client.as_ref().unwrap();
        let buckets = client.load_all_buckets().await;
        match buckets {
            Ok(buckets) => {
                self.app_objects.set_items(Vec::new(), buckets);
                self.app_view_state.view_state = ViewState::Default;
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e.msg)).unwrap();
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn current_key_string(&self) -> String {
        format!(" {} ", self.current_keys.join(" / "))
    }

    fn current_bucket(&self) -> String {
        self.current_keys[0].clone()
    }

    fn current_bucket_opt(&self) -> Option<&String> {
        self.current_keys.get(0)
    }

    fn current_object_prefix(&self) -> String {
        let mut prefix = String::new();
        for key in &self.current_keys[1..] {
            prefix.push_str(key);
            prefix.push('/');
        }
        prefix
    }

    pub fn current_items(&self) -> Vec<Item> {
        self.app_objects.get_items(&self.current_keys)
    }

    fn current_items_len(&self) -> usize {
        self.app_objects.get_items_len(&self.current_keys)
    }

    fn get_current_selected(&self) -> Option<&Item> {
        self.app_view_state
            .current_list_state
            .selected()
            .and_then(|i| self.app_objects.get_item(&self.current_keys, i))
    }

    pub fn get_current_file_detail(&self) -> Option<&FileDetail> {
        self.get_current_selected().and_then(|selected| {
            if let Item::File { name, .. } = selected {
                let bucket = &self.current_bucket();
                let prefix = &self.current_object_prefix();
                let key = &self.object_detail_map_key(bucket, prefix, name);
                self.app_objects.get_object_detail(key)
            } else {
                None
            }
        })
    }

    pub fn get_current_file_versions(&self) -> Option<&Vec<FileVersion>> {
        self.get_current_selected().and_then(|selected| {
            if let Item::File { name, .. } = selected {
                let bucket = &self.current_bucket();
                let prefix = &self.current_object_prefix();
                let key = &self.object_detail_map_key(bucket, prefix, name);
                self.app_objects.get_object_versions(key)
            } else {
                None
            }
        })
    }

    pub fn select_next(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::ObjectDetail(_) | ViewState::Help => {}
            ViewState::Default => {
                if let Some(i) = self.app_view_state.current_list_state.selected() {
                    let len = self.current_items_len();
                    let i = if len == 0 || i >= len - 1 { 0 } else { i + 1 };
                    self.app_view_state.current_list_state.select(Some(i));
                };
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::ObjectDetail(_) | ViewState::Help => {}
            ViewState::Default => {
                if let Some(i) = self.app_view_state.current_list_state.selected() {
                    let len = self.current_items_len();
                    let i = if len == 0 {
                        0
                    } else if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    };
                    self.app_view_state.current_list_state.select(Some(i));
                };
            }
        }
    }

    pub fn select_first(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::ObjectDetail(_) | ViewState::Help => {}
            ViewState::Default => {
                let i = 0;
                self.app_view_state.current_list_state.select(Some(i));
            }
        }
    }

    pub fn select_last(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::ObjectDetail(_) | ViewState::Help => {}
            ViewState::Default => {
                let i = self.current_items_len() - 1;
                self.app_view_state.current_list_state.select(Some(i));
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::ObjectDetail(_) | ViewState::Help => {}
            ViewState::Default => {
                if let Some(selected) = self.get_current_selected() {
                    if let Item::File { .. } = selected {
                        if self.exists_current_object_detail() {
                            self.app_view_state.view_state =
                                ViewState::ObjectDetail(FileDetailViewState::Detail);
                        } else {
                            self.tx.send(AppEventType::LoadObject).unwrap();
                            self.app_view_state.is_loading = true;
                        }
                    } else {
                        self.current_keys.push(selected.name().to_owned());
                        self.app_view_state.current_list_state.select(Some(0));

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
        let bucket = &self.current_bucket();
        let prefix = &self.current_object_prefix();
        match self.get_current_selected() {
            Some(selected) => {
                let map_key = &self.object_detail_map_key(bucket, prefix, selected.name());
                self.app_objects.exists_object_details(map_key)
            }
            None => false,
        }
    }

    fn exists_current_objects(&self) -> bool {
        self.app_objects.exists_item(&self.current_keys)
    }

    pub fn move_up(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Help => {}
            ViewState::Default => {
                self.current_keys.pop();
                self.app_view_state.current_list_state.select(Some(0));
            }
            ViewState::ObjectDetail(_) => {
                self.app_view_state.view_state = ViewState::Default;
            }
        }
    }

    pub async fn load_objects(&mut self) {
        let bucket = &self.current_bucket();
        let prefix = &self.current_object_prefix();
        let client = self.client.as_ref().unwrap();
        let items = client.load_objects(bucket, prefix).await;
        match items {
            Ok(items) => {
                self.app_objects.set_items(self.current_keys.clone(), items);
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e.msg)).unwrap();
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub async fn load_object(&mut self) {
        if let Some(Item::File {
            name, size_byte, ..
        }) = self.get_current_selected()
        {
            let bucket = &self.current_bucket();
            let prefix = &self.current_object_prefix();
            let key = &format!("{}{}", prefix, name);

            let map_key = &self.object_detail_map_key(bucket, prefix, name);

            let client = self.client.as_ref().unwrap();
            let detail = client
                .load_object_detail(bucket, key, name, *size_byte)
                .await;
            let versions = client.load_object_versions(bucket, key).await;

            match (detail, versions) {
                (Ok(detail), Ok(versions)) => {
                    self.app_objects
                        .set_object_details(map_key, detail, versions);

                    self.app_view_state.view_state =
                        ViewState::ObjectDetail(FileDetailViewState::Detail);
                }
                (Err(e), _) => {
                    self.tx.send(AppEventType::Error(e.msg)).unwrap();
                }
                (_, Err(e)) => {
                    self.tx.send(AppEventType::Error(e.msg)).unwrap();
                }
            }
        }
        self.app_view_state.is_loading = false;
    }

    fn object_detail_map_key(&self, bucket: &String, prefix: &String, name: &String) -> String {
        format!("{}/{}{}", bucket, prefix, name)
    }

    pub fn select_tabs(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Default | ViewState::Help => {}
            ViewState::ObjectDetail(vs) => match vs {
                FileDetailViewState::Detail => {
                    self.app_view_state.view_state =
                        ViewState::ObjectDetail(FileDetailViewState::Version);
                }
                FileDetailViewState::Version => {
                    self.app_view_state.view_state =
                        ViewState::ObjectDetail(FileDetailViewState::Detail);
                }
            },
        }
    }

    pub fn toggle_help(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing => {}
            ViewState::Help => {
                self.app_view_state.view_state = self.app_view_state.before_view_state.unwrap();
                self.app_view_state.before_view_state = None;
            }
            ViewState::Default | ViewState::ObjectDetail(_) => {
                self.app_view_state.before_view_state = Some(self.app_view_state.view_state);
                self.app_view_state.view_state = ViewState::Help;
            }
        }
    }

    pub fn download(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Default | ViewState::Help => {}
            ViewState::ObjectDetail(_) => {
                self.tx.send(AppEventType::DownloadObject).unwrap();
                self.app_view_state.is_loading = true;
            }
        }
    }

    pub async fn download_object(&mut self) {
        if let Some(Item::File { name, .. }) = self.get_current_selected() {
            let client = self.client.as_ref().unwrap();
            let bucket = &self.current_bucket();
            let prefix = &self.current_object_prefix();
            let key = &format!("{}{}", prefix, name);
            let bytes = client.download_object(bucket, key).await;

            let config = self.config.as_ref().unwrap();
            let path = config.download_file_path(name);
            let result = bytes.and_then(|bs| save_binary(&path, &bs));
            if let Err(e) = result {
                self.tx.send(AppEventType::Error(e.msg)).unwrap();
            } else {
                let msg = format!("Download completed successfully: {}", name);
                self.tx.send(AppEventType::Info(msg)).unwrap();
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn open_management_console(&self) {
        let client = self.client.as_ref().unwrap();
        let bucket = self.current_bucket_opt();

        let result = match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Help => Ok(()),
            ViewState::Default => match bucket {
                Some(bucket) => {
                    let prefix = self.current_object_prefix();
                    client.open_management_console_list(bucket, &prefix)
                }
                None => client.open_management_console_buckets(),
            },
            ViewState::ObjectDetail(_) => {
                if let Some(Item::File { name, .. }) = self.get_current_selected() {
                    let prefix = self.current_object_prefix();
                    client.open_management_console_object(bucket.unwrap(), &prefix, name)
                } else {
                    Err(AppError::msg("Failed to get current selected item"))
                }
            }
        };
        if let Err(e) = result {
            self.tx.send(AppEventType::Error(e.msg)).unwrap();
        }
    }
}

#[derive(Clone, Debug)]
pub enum Item {
    Bucket {
        name: String,
    },
    Dir {
        name: String,
        paths: Vec<String>,
    },
    File {
        name: String,
        paths: Vec<String>,
        size_byte: i64,
        last_modified: DateTime<Local>,
    },
}

impl Item {
    fn name(&self) -> &String {
        match self {
            Item::Bucket { name } => name,
            Item::Dir { name, .. } => name,
            Item::File { name, .. } => name,
        }
    }
}

pub struct FileDetail {
    pub name: String,
    pub size_byte: i64,
    pub last_modified: DateTime<Local>,
    pub e_tag: String,
    pub content_type: String,
}

pub struct FileVersion {
    pub version_id: String,
    pub size_byte: i64,
    pub last_modified: DateTime<Local>,
    pub is_latest: bool,
}

pub struct AppError<'a> {
    pub msg: String,
    pub e: Option<Box<dyn Error + Send + 'a>>,
}

impl<'a> AppError<'a> {
    pub fn new<E: Error + Send + 'a>(msg: impl Into<String>, e: E) -> AppError<'a> {
        AppError {
            msg: msg.into(),
            e: Some(Box::new(e)),
        }
    }

    pub fn msg(msg: impl Into<String>) -> AppError<'a> {
        AppError {
            msg: msg.into(),
            e: None,
        }
    }

    pub fn error<E: Error + Send + 'a>(e: E) -> AppError<'a> {
        AppError {
            msg: e.to_string(),
            e: Some(Box::new(e)),
        }
    }
}
