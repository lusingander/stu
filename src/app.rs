use std::{
    collections::HashMap,
    sync::{mpsc, Arc},
};
use tokio::spawn;

use crate::{
    client::Client,
    config::Config,
    error::AppError,
    event::AppEventType,
    file::{save_binary, save_error_log},
    item::{FileDetail, FileVersion, Item},
};

pub struct App {
    pub app_view_state: AppViewState,
    app_objects: AppObjects,
    current_keys: Vec<String>,
    client: Option<Arc<Client>>,
    config: Option<Config>,
    tx: mpsc::Sender<AppEventType>,
}

pub struct AppViewState {
    pub list_selected: usize,
    pub list_offset: usize,
    list_height: usize,
    pub view_state: ViewState,
    pub before_view_state: Option<ViewState>,
    pub notification: Notification,
    pub is_loading: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ViewState {
    Initializing,
    List,
    Detail(DetailViewState),
    Help,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum DetailViewState {
    Detail = 0,
    Version = 1,
}

pub enum Notification {
    None,
    Info(String),
    Error(String),
}

impl AppViewState {
    fn new(height: usize) -> AppViewState {
        AppViewState {
            list_selected: 0,
            list_offset: 0,
            list_height: height - 3 /* header */ - 2 /* footer */ - 2, /* list area border */
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
    pub fn new(tx: mpsc::Sender<AppEventType>, height: usize) -> App {
        App {
            app_view_state: AppViewState::new(height),
            app_objects: AppObjects::new(),
            current_keys: Vec::new(),
            client: None,
            config: None,
            tx,
        }
    }

    pub async fn initialize(&mut self, config: Config, client: Client) {
        self.config = Some(config);
        self.client = Some(Arc::new(client));

        let client = self.client.as_ref().unwrap();
        let buckets = client.load_all_buckets().await;
        match buckets {
            Ok(buckets) => {
                self.app_objects.set_items(Vec::new(), buckets);
                self.app_view_state.view_state = ViewState::List;
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
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
        let i = self.app_view_state.list_selected;
        self.app_objects.get_item(&self.current_keys, i)
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
            ViewState::Initializing | ViewState::Detail(_) | ViewState::Help => {}
            ViewState::List => {
                let current_selected = self.app_view_state.list_selected;
                let len = self.current_items_len();
                if len == 0 || current_selected >= len - 1 {
                    self.app_view_state.list_selected = 0;
                    self.app_view_state.list_offset = 0;
                } else {
                    self.app_view_state.list_selected = current_selected + 1;

                    if current_selected - self.app_view_state.list_offset
                        == self.app_view_state.list_height - 1
                    {
                        self.app_view_state.list_offset += 1;
                    }
                };
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Detail(_) | ViewState::Help => {}
            ViewState::List => {
                let current_selected = self.app_view_state.list_selected;
                let len = self.current_items_len();
                if len == 0 {
                    self.app_view_state.list_selected = 0;
                    self.app_view_state.list_offset = 0;
                } else if current_selected == 0 {
                    self.app_view_state.list_selected = len - 1;

                    if self.app_view_state.list_height < len {
                        self.app_view_state.list_offset = len - self.app_view_state.list_height;
                    }
                } else {
                    self.app_view_state.list_selected = current_selected - 1;

                    if current_selected - self.app_view_state.list_offset == 0 {
                        self.app_view_state.list_offset -= 1;
                    }
                };
            }
        }
    }

    pub fn select_first(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Detail(_) | ViewState::Help => {}
            ViewState::List => {
                self.app_view_state.list_selected = 0;
                self.app_view_state.list_offset = 0;
            }
        }
    }

    pub fn select_last(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Detail(_) | ViewState::Help => {}
            ViewState::List => {
                let len = self.current_items_len();
                self.app_view_state.list_selected = len - 1;
                if self.app_view_state.list_height < len {
                    self.app_view_state.list_offset = len - self.app_view_state.list_height;
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Detail(_) | ViewState::Help => {}
            ViewState::List => {
                if let Some(selected) = self.get_current_selected() {
                    if let Item::File { .. } = selected {
                        if self.exists_current_object_detail() {
                            self.app_view_state.view_state =
                                ViewState::Detail(DetailViewState::Detail);
                        } else {
                            self.tx.send(AppEventType::LoadObject).unwrap();
                            self.app_view_state.is_loading = true;
                        }
                    } else {
                        self.current_keys.push(selected.name().to_owned());
                        self.app_view_state.list_selected = 0;
                        self.app_view_state.list_offset = 0;

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
            ViewState::Initializing => {}
            ViewState::List => {
                let key = self.current_keys.pop();
                if key.is_some() {
                    self.app_view_state.list_selected = 0;
                    self.app_view_state.list_offset = 0;
                }
            }
            ViewState::Detail(_) => {
                self.app_view_state.view_state = ViewState::List;
            }
            ViewState::Help => {
                self.toggle_help();
            }
        }
    }

    pub fn load_objects(&self) {
        let bucket = self.current_bucket();
        let prefix = self.current_object_prefix();
        let client = self.client.as_ref().unwrap().clone();
        let tx = self.tx.clone();
        spawn(async move {
            let result = client.load_objects(&bucket, &prefix).await;
            tx.send(AppEventType::CompleteLoadObjects(result)).unwrap();
        });
    }

    pub fn complete_load_objects(&mut self, result: Result<Vec<Item>, AppError>) {
        match result {
            Ok(items) => {
                self.app_objects.set_items(self.current_keys.clone(), items);
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn load_object(&self) {
        if let Some(Item::File {
            name, size_byte, ..
        }) = self.get_current_selected()
        {
            let name = name.clone();
            let size_byte = *size_byte;

            let bucket = self.current_bucket();
            let prefix = self.current_object_prefix();
            let key = format!("{}{}", prefix, name);

            let map_key = self.object_detail_map_key(&bucket, &prefix, &name);

            let client = self.client.as_ref().unwrap().clone();
            let tx = self.tx.clone();

            spawn(async move {
                let detail = client
                    .load_object_detail(&bucket, &key, &name, size_byte)
                    .await;
                let versions = client.load_object_versions(&bucket, &key).await;
                let result = detail.and_then(|d| versions.map(|v| (d, v, map_key)));
                tx.send(AppEventType::CompleteLoadObject(result)).unwrap();
            });
        }
    }

    pub fn complete_load_object(
        &mut self,
        result: Result<(FileDetail, Vec<FileVersion>, String), AppError>,
    ) {
        match result {
            Ok((detail, versions, map_key)) => {
                self.app_objects
                    .set_object_details(&map_key, detail, versions);
                self.app_view_state.view_state = ViewState::Detail(DetailViewState::Detail);
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
            }
        }
        self.app_view_state.is_loading = false;
    }

    fn object_detail_map_key(&self, bucket: &String, prefix: &String, name: &String) -> String {
        format!("{}/{}{}", bucket, prefix, name)
    }

    pub fn select_tabs(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::List | ViewState::Help => {}
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
        match self.app_view_state.view_state {
            ViewState::Initializing => {}
            ViewState::Help => {
                self.app_view_state.view_state = self.app_view_state.before_view_state.unwrap();
                self.app_view_state.before_view_state = None;
            }
            ViewState::List | ViewState::Detail(_) => {
                self.app_view_state.before_view_state = Some(self.app_view_state.view_state);
                self.app_view_state.view_state = ViewState::Help;
            }
        }
    }

    pub fn download(&mut self) {
        match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::List | ViewState::Help => {}
            ViewState::Detail(_) => {
                self.tx.send(AppEventType::DownloadObject).unwrap();
                self.app_view_state.is_loading = true;
            }
        }
    }

    pub fn download_object(&mut self) {
        if let Some(Item::File { name, .. }) = self.get_current_selected() {
            let bucket = self.current_bucket();
            let prefix = self.current_object_prefix();
            let key = format!("{}{}", prefix, name);

            let config = self.config.as_ref().unwrap();
            let path = config.download_file_path(name);

            let client = self.client.as_ref().unwrap().clone();
            let tx = self.tx.clone();

            spawn(async move {
                let bytes = client.download_object(&bucket, &key).await;
                let result = bytes.map(|bs| (bs, path));
                tx.send(AppEventType::CompleteDownloadObject(result))
                    .unwrap();
            });
        }
    }

    pub fn complete_download_object(&mut self, result: Result<(Vec<u8>, String), AppError>) {
        let result = result.and_then(|(bs, path)| save_binary(&path, &bs).map(|_| path));
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

    pub fn save_error(&self, e: &AppError) {
        let config = self.config.as_ref().unwrap();
        // cause panic if save errors
        let path = config.error_log_path().unwrap();
        save_error_log(&path, e).unwrap();
    }

    pub fn open_management_console(&self) {
        let client = self.client.as_ref().unwrap();
        let bucket = self.current_bucket_opt();

        let result = match self.app_view_state.view_state {
            ViewState::Initializing | ViewState::Help => Ok(()),
            ViewState::List => match bucket {
                Some(bucket) => {
                    let prefix = self.current_object_prefix();
                    client.open_management_console_list(bucket, &prefix)
                }
                None => client.open_management_console_buckets(),
            },
            ViewState::Detail(_) => {
                if let Some(Item::File { name, .. }) = self.get_current_selected() {
                    let prefix = self.current_object_prefix();
                    client.open_management_console_object(bucket.unwrap(), &prefix, name)
                } else {
                    Err(AppError::msg("Failed to get current selected item"))
                }
            }
        };
        if let Err(e) = result {
            self.tx.send(AppEventType::Error(e)).unwrap();
        }
    }
}
