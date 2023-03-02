use chrono::{DateTime, Local};
use std::{collections::HashMap, sync::mpsc};
use tui::widgets::ListState;

use crate::{client::Client, event::AppEventType};

pub struct App {
    pub current_list_state: ListState,
    pub view_state: ViewState,
    pub before_view_state: Option<ViewState>,
    pub file_detail_view_state: FileDetailViewState,
    pub is_loading: bool,
    current_keys: Vec<String>,
    items_map: HashMap<Vec<String>, Vec<Item>>,
    detail_map: HashMap<String, FileDetail>,
    versions_map: HashMap<String, Vec<FileVersion>>,
    error_msg: Option<String>,
    client: Option<Client>,
    tx: mpsc::Sender<AppEventType>,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ViewState {
    Initializing,
    Default,
    ObjectDetail,
    Help,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FileDetailViewState {
    Detail = 0,
    Version = 1,
}

impl App {
    pub fn new(tx: mpsc::Sender<AppEventType>) -> App {
        let mut current_list_state = ListState::default();
        current_list_state.select(Some(0));

        App {
            current_list_state,
            view_state: ViewState::Initializing,
            file_detail_view_state: FileDetailViewState::Detail,
            is_loading: true,
            current_keys: Vec::new(),
            items_map: HashMap::new(),
            detail_map: HashMap::new(),
            versions_map: HashMap::new(),
            error_msg: None,
            before_view_state: None,
            client: None,
            tx,
        }
    }

    pub async fn initialize(&mut self, client: Client) {
        self.client = Some(client);

        let client = self.client.as_ref().unwrap();
        let buckets = client.load_all_buckets().await;
        match buckets {
            Ok(buckets) => {
                self.items_map.insert(Vec::new(), buckets);
                self.view_state = ViewState::Default;
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
            }
        }
        self.is_loading = false;
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
        self.items_map
            .get(&self.current_keys)
            .unwrap_or(&Vec::new())
            .to_vec()
    }

    fn current_items_len(&self) -> usize {
        self.items_map
            .get(&self.current_keys)
            .unwrap_or(&Vec::new())
            .len()
    }

    fn get_from_current_items(&self, idx: usize) -> Option<&Item> {
        self.items_map
            .get(&self.current_keys)
            .and_then(|items| items.get(idx))
    }

    fn get_current_selected(&self) -> Option<&Item> {
        self.current_list_state
            .selected()
            .and_then(|i| self.get_from_current_items(i))
    }

    pub fn get_current_file_detail(&self) -> Option<&FileDetail> {
        self.get_current_selected().and_then(|selected| {
            if let Item::File { name, .. } = selected {
                let bucket = &self.current_bucket();
                let prefix = &self.current_object_prefix();
                let key = &self.object_detail_map_key(bucket, prefix, name);
                self.detail_map.get(key)
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
                self.versions_map.get(key)
            } else {
                None
            }
        })
    }

    pub fn select_next(&mut self) {
        match self.view_state {
            ViewState::Initializing | ViewState::ObjectDetail | ViewState::Help => {}
            ViewState::Default => {
                if let Some(i) = self.current_list_state.selected() {
                    let len = self.current_items_len();
                    let i = if len == 0 || i >= len - 1 { 0 } else { i + 1 };
                    self.current_list_state.select(Some(i));
                };
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.view_state {
            ViewState::Initializing | ViewState::ObjectDetail | ViewState::Help => {}
            ViewState::Default => {
                if let Some(i) = self.current_list_state.selected() {
                    let len = self.current_items_len();
                    let i = if len == 0 {
                        0
                    } else if i == 0 {
                        len - 1
                    } else {
                        i - 1
                    };
                    self.current_list_state.select(Some(i));
                };
            }
        }
    }

    pub fn select_first(&mut self) {
        match self.view_state {
            ViewState::Initializing | ViewState::ObjectDetail | ViewState::Help => {}
            ViewState::Default => {
                let i = 0;
                self.current_list_state.select(Some(i));
            }
        }
    }

    pub fn select_last(&mut self) {
        match self.view_state {
            ViewState::Initializing | ViewState::ObjectDetail | ViewState::Help => {}
            ViewState::Default => {
                let i = self.current_items_len() - 1;
                self.current_list_state.select(Some(i));
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.view_state {
            ViewState::Initializing | ViewState::ObjectDetail | ViewState::Help => {}
            ViewState::Default => {
                if let Some(selected) = self.get_current_selected() {
                    if let Item::File { .. } = selected {
                        if self.exists_current_object_detail() {
                            self.view_state = ViewState::ObjectDetail;
                            self.file_detail_view_state = FileDetailViewState::Detail;
                        } else {
                            self.tx.send(AppEventType::LoadObject).unwrap();
                            self.is_loading = true;
                        }
                    } else {
                        self.current_keys.push(selected.name().to_owned());
                        self.current_list_state.select(Some(0));

                        if !self.exists_current_objects() {
                            self.tx.send(AppEventType::LoadObjects).unwrap();
                            self.is_loading = true;
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
                self.detail_map.contains_key(map_key) && self.versions_map.contains_key(map_key)
            }
            None => false,
        }
    }

    fn exists_current_objects(&self) -> bool {
        self.items_map.contains_key(&self.current_keys)
    }

    pub fn move_up(&mut self) {
        match self.view_state {
            ViewState::Initializing | ViewState::Help => {}
            ViewState::Default => {
                self.current_keys.pop();
                self.current_list_state.select(Some(0));
            }
            ViewState::ObjectDetail => {
                self.view_state = ViewState::Default;
                self.file_detail_view_state = FileDetailViewState::Detail;
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
                self.items_map.insert(self.current_keys.clone(), items);
            }
            Err(e) => {
                self.tx.send(AppEventType::Error(e)).unwrap();
            }
        }
        self.is_loading = false;
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
                    self.detail_map.insert(map_key.to_owned(), detail);
                    self.versions_map.insert(map_key.to_owned(), versions);

                    self.view_state = ViewState::ObjectDetail;
                    self.file_detail_view_state = FileDetailViewState::Detail;
                }
                (Err(e), _) => {
                    self.tx.send(AppEventType::Error(e)).unwrap();
                }
                (_, Err(e)) => {
                    self.tx.send(AppEventType::Error(e)).unwrap();
                }
            }
        }
        self.is_loading = false;
    }

    fn object_detail_map_key(&self, bucket: &String, prefix: &String, name: &String) -> String {
        format!("{}/{}{}", bucket, prefix, name)
    }

    pub fn select_tabs(&mut self) {
        match self.view_state {
            ViewState::Initializing | ViewState::Default | ViewState::Help => {}
            ViewState::ObjectDetail => match self.file_detail_view_state {
                FileDetailViewState::Detail => {
                    self.file_detail_view_state = FileDetailViewState::Version;
                }
                FileDetailViewState::Version => {
                    self.file_detail_view_state = FileDetailViewState::Detail;
                }
            },
        }
    }

    pub fn toggle_help(&mut self) {
        match self.view_state {
            ViewState::Initializing => {}
            ViewState::Help => {
                self.view_state = self.before_view_state.unwrap();
                self.before_view_state = None;
            }
            ViewState::Default | ViewState::ObjectDetail => {
                self.before_view_state = Some(self.view_state);
                self.view_state = ViewState::Help;
            }
        }
    }

    pub fn open_management_console(&self) {
        let client = self.client.as_ref().unwrap();
        let bucket = self.current_bucket_opt();

        let result = match self.view_state {
            ViewState::Initializing | ViewState::Help => Ok(()),
            ViewState::Default => match bucket {
                Some(bucket) => {
                    let prefix = self.current_object_prefix();
                    client.open_management_console_list(bucket, &prefix)
                }
                None => client.open_management_console_buckets(),
            },
            ViewState::ObjectDetail => {
                if let Some(Item::File { name, .. }) = self.get_current_selected() {
                    let prefix = self.current_object_prefix();
                    client.open_management_console_object(bucket.unwrap(), &prefix, name)
                } else {
                    Err("Failed to get current selected item".to_string())
                }
            }
        };
        if let Err(e) = result {
            self.tx.send(AppEventType::Error(e)).unwrap();
        }
    }

    pub fn set_error_msg(&mut self, msg: String) {
        self.error_msg = Some(msg);
    }

    pub fn clear_error_msg(&mut self) {
        self.error_msg = None;
    }

    pub fn has_error(&self) -> bool {
        self.error_msg.is_some()
    }

    pub fn get_error_msg(&self) -> &String {
        self.error_msg.as_ref().unwrap()
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
