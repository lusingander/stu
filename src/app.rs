use chrono::{DateTime, Local};
use std::collections::HashMap;
use tui::widgets::ListState;

use crate::client::Client;

pub struct App {
    pub current_list_state: ListState,
    current_keys: Vec<String>,
    item_map: HashMap<Vec<String>, Vec<Item>>,
    client: Client,
}

impl App {
    pub async fn new(client: Client) -> App {
        let mut current_list_state = ListState::default();
        current_list_state.select(Some(0));

        let buckets = client.load_all_buckets().await;
        let mut item_map = HashMap::new();
        item_map.insert(Vec::new(), buckets);

        App {
            current_keys: Vec::new(),
            current_list_state,
            item_map,
            client,
        }
    }

    pub fn current_key_string(&self) -> String {
        format!(" {} ", self.current_keys.join(" / "))
    }

    fn current_bucket(&self) -> String {
        self.current_keys[0].clone()
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
        self.item_map.get(&self.current_keys).unwrap().to_vec()
    }

    fn current_items_len(&self) -> usize {
        self.item_map.get(&self.current_keys).unwrap().len()
    }

    fn get_from_current_items(&self, idx: usize) -> Option<&Item> {
        self.item_map.get(&self.current_keys).unwrap().get(idx)
    }

    pub fn select_next(&mut self) {
        if let Some(i) = self.current_list_state.selected() {
            let i = if i >= self.current_items_len() - 1 {
                0
            } else {
                i + 1
            };
            self.current_list_state.select(Some(i));
        };
    }

    pub fn select_prev(&mut self) {
        if let Some(i) = self.current_list_state.selected() {
            let i = if i == 0 {
                self.current_items_len() - 1
            } else {
                i - 1
            };
            self.current_list_state.select(Some(i));
        };
    }

    pub fn select_first(&mut self) {
        let i = 0;
        self.current_list_state.select(Some(i));
    }

    pub fn select_last(&mut self) {
        let i = self.current_items_len() - 1;
        self.current_list_state.select(Some(i));
    }

    pub async fn move_down(&mut self) {
        let selected = self
            .current_list_state
            .selected()
            .and_then(|i| self.get_from_current_items(i))
            .unwrap();

        if let Item::File { .. } = selected {
        } else {
            self.current_keys.push(selected.name().to_owned());
            self.load_objects().await;

            self.current_list_state.select(Some(0));
        }
    }

    pub async fn move_up(&mut self) {
        self.current_keys.pop();

        self.current_list_state.select(Some(0));
    }

    async fn load_objects(&mut self) {
        let items = self
            .client
            .load_objects(&self.current_bucket(), &self.current_object_prefix())
            .await;
        self.item_map.insert(self.current_keys.clone(), items);
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
