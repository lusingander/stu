use std::sync::Arc;
use tokio::spawn;

use crate::{
    client::Client,
    config::Config,
    error::{AppError, Result},
    event::{
        AppEventType, CompleteDownloadObjectResult, CompleteInitializeResult,
        CompleteLoadObjectResult, CompleteLoadObjectsResult, CompletePreviewObjectResult, Sender,
    },
    file::{copy_to_clipboard, save_binary, save_error_log},
    if_match,
    object::{AppObjects, BucketItem, FileDetail, ObjectItem, ObjectKey, RawObject},
    pages::page::{Page, PageStack},
};

#[derive(Debug)]
pub enum Notification {
    None,
    Info(String),
    Success(String),
    Warn(String),
    Error(String),
}

#[derive(Debug)]
pub struct AppViewState {
    pub notification: Notification,
    pub is_loading: bool,

    width: usize,
    height: usize,
}

impl AppViewState {
    fn new(width: usize, height: usize) -> AppViewState {
        AppViewState {
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

#[derive(Debug)]
pub struct App {
    pub app_view_state: AppViewState,
    pub page_stack: PageStack,
    app_objects: AppObjects,
    client: Option<Arc<Client>>,
    config: Config,
    tx: Sender,
}

impl App {
    pub fn new(config: Config, tx: Sender, width: usize, height: usize) -> App {
        App {
            app_view_state: AppViewState::new(width, height),
            app_objects: AppObjects::default(),
            page_stack: PageStack::new(tx.clone()),
            client: None,
            config,
            tx,
        }
    }

    pub fn initialize(&mut self, client: Client, bucket: Option<String>) {
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

                let bucket_list_page = Page::of_bucket_list(self.bucket_items(), self.tx.clone());
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

    fn current_bucket(&self) -> String {
        let bucket_page = self.page_stack.head().as_bucket_list();
        bucket_page.current_selected_item().name.clone()
    }

    fn current_path(&self) -> Vec<&str> {
        self.page_stack
            .iter()
            .filter_map(|page| if_match! { page: Page::ObjectList(p) => p })
            .map(|page| page.current_selected_item())
            .filter_map(|item| if_match! { item: ObjectItem::Dir { name, .. } => name.as_str() })
            .collect()
    }

    fn current_object_prefix(&self) -> String {
        let mut prefix = String::new();
        for key in &self.current_path() {
            prefix.push_str(key);
            prefix.push('/');
        }
        prefix
    }

    fn current_object_key(&self) -> ObjectKey {
        ObjectKey {
            bucket_name: self.current_bucket(),
            object_path: self.current_path().iter().map(|s| s.to_string()).collect(),
        }
    }

    fn current_object_key_with_name(&self, name: String) -> ObjectKey {
        let mut object_path: Vec<String> =
            self.current_path().iter().map(|s| s.to_string()).collect();
        object_path.push(name);
        ObjectKey {
            bucket_name: self.current_bucket(),
            object_path,
        }
    }

    fn bucket_items(&self) -> Vec<BucketItem> {
        self.app_objects.get_bucket_items()
    }

    fn current_object_items(&self) -> Option<Vec<ObjectItem>> {
        self.app_objects
            .get_object_items(&self.current_object_key())
    }

    pub fn bucket_list_move_down(&mut self) {
        if let Some(current_object_items) = self.current_object_items() {
            // object list has been already loaded
            let object_list_page = Page::of_object_list(current_object_items, self.tx.clone());
            self.page_stack.push(object_list_page);
        } else {
            self.tx.send(AppEventType::LoadObjects);
            self.app_view_state.is_loading = true;
        }
    }

    pub fn object_list_move_down(&mut self) {
        let object_page = self.page_stack.current_page().as_object_list();
        let selected = object_page.current_selected_item().to_owned();

        match selected {
            ObjectItem::File { name, .. } => {
                let current_object_key = &self.current_object_key_with_name(name.to_string());
                let detail = self.app_objects.get_object_detail(current_object_key);
                let versions = self.app_objects.get_object_versions(current_object_key);

                if let (Some(detail), Some(versions)) = (detail, versions) {
                    // object has been already loaded
                    let object_detail_page = Page::of_object_detail(
                        detail.clone(),
                        versions.clone(),
                        object_page.object_list(),
                        object_page.list_state(),
                        self.tx.clone(),
                    );
                    self.page_stack.push(object_detail_page);
                } else {
                    self.tx.send(AppEventType::LoadObject);
                    self.app_view_state.is_loading = true;
                }
            }
            ObjectItem::Dir { .. } => {
                if let Some(current_object_items) = self.current_object_items() {
                    // object list has been already loaded
                    let object_list_page =
                        Page::of_object_list(current_object_items, self.tx.clone());
                    self.page_stack.push(object_list_page);
                } else {
                    self.tx.send(AppEventType::LoadObjects);
                    self.app_view_state.is_loading = true;
                }
            }
        }
    }

    pub fn object_list_move_up(&mut self) {
        if self.page_stack.len() == 2 /* bucket list and object list */ && self.bucket_items().len() == 1
        {
            return;
        }
        self.page_stack.pop();
    }

    pub fn back_to_bucket_list(&mut self) {
        if self.bucket_items().len() == 1 {
            return;
        }
        self.page_stack.clear();
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
                    .set_object_items(self.current_object_key().to_owned(), items.clone());

                let object_list_page = Page::of_object_list(items, self.tx.clone());
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

                let object_page = self.page_stack.current_page().as_object_list();

                let object_detail_page = Page::of_object_detail(
                    *detail.clone(),
                    versions.clone(),
                    object_page.object_list(),
                    object_page.list_state(),
                    self.tx.clone(),
                );
                self.page_stack.push(object_detail_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn open_help(&mut self) {
        let helps = match self.page_stack.current_page() {
            Page::Initializing(_) | Page::Help(_) => {
                return;
            }
            Page::BucketList(page) => page.helps(),
            Page::ObjectList(page) => page.helps(),
            Page::ObjectDetail(page) => page.helps(),
            Page::ObjectPreview(page) => page.helps(),
        };
        let help_page = Page::of_help(helps, self.tx.clone());
        self.page_stack.push(help_page);
    }

    pub fn close_current_page(&mut self) {
        self.page_stack.pop();
    }

    pub fn detail_download_object(&mut self, file_detail: FileDetail, version_id: Option<String>) {
        self.tx
            .send(AppEventType::DownloadObject(file_detail, version_id));
        self.app_view_state.is_loading = true;
    }

    pub fn preview_download_object(&self, obj: RawObject, path: String) {
        let result = CompleteDownloadObjectResult::new(Ok(obj), path);
        self.tx.send(AppEventType::CompleteDownloadObject(result));
    }

    pub fn open_preview(&mut self, file_detail: FileDetail, version_id: Option<String>) {
        self.tx
            .send(AppEventType::PreviewObject(file_detail, version_id));
        self.app_view_state.is_loading = true;
    }

    pub fn download_object(&self, file_detail: FileDetail, version_id: Option<String>) {
        let object_name = file_detail.name;
        let size_byte = file_detail.size_byte;

        self.download_object_and(
            &object_name,
            size_byte,
            None,
            version_id,
            |tx, obj, path| {
                let result = CompleteDownloadObjectResult::new(obj, path);
                tx.send(AppEventType::CompleteDownloadObject(result));
            },
        )
    }

    pub fn download_object_as(
        &self,
        file_detail: FileDetail,
        input: String,
        version_id: Option<String>,
    ) {
        let object_name = file_detail.name;
        let size_byte = file_detail.size_byte;

        self.download_object_and(
            &object_name,
            size_byte,
            Some(&input),
            version_id,
            |tx, obj, path| {
                let result = CompleteDownloadObjectResult::new(obj, path);
                tx.send(AppEventType::CompleteDownloadObject(result));
            },
        )
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

    pub fn preview_object(&self, file_detail: FileDetail, version_id: Option<String>) {
        let object_name = file_detail.name.clone();
        let size_byte = file_detail.size_byte;

        self.download_object_and(
            &object_name,
            size_byte,
            None,
            version_id.clone(),
            |tx, obj, path| {
                let result = CompletePreviewObjectResult::new(obj, file_detail, version_id, path);
                tx.send(AppEventType::CompletePreviewObject(result));
            },
        )
    }

    pub fn complete_preview_object(&mut self, result: Result<CompletePreviewObjectResult>) {
        match result {
            Ok(CompletePreviewObjectResult {
                obj,
                file_detail,
                file_version_id,
                path,
            }) => {
                let object_preview_page = Page::of_object_preview(
                    file_detail,
                    file_version_id,
                    obj,
                    path,
                    self.config.preview.clone(),
                    self.tx.clone(),
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
        version_id: Option<String>,
        f: F,
    ) where
        F: FnOnce(Sender, Result<RawObject>, String) + Send + 'static,
    {
        let bucket = self.current_bucket();
        let prefix = self.current_object_prefix();
        let key = format!("{}{}", prefix, object_name);

        let path = self
            .config
            .download_file_path(save_file_name.unwrap_or(object_name));

        let (client, tx) = self.unwrap_client_tx();
        let loading = self.handle_loading_size(size_byte, tx.clone());
        spawn(async move {
            let obj = client
                .download_object(&bucket, &key, version_id, size_byte, loading)
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
        let (client, _) = self.unwrap_client_tx();
        let result = client.open_management_console_buckets();
        if let Err(e) = result {
            self.tx.send(AppEventType::NotifyError(e));
        }
    }

    pub fn object_list_open_management_console(&self) {
        let (client, _) = self.unwrap_client_tx();
        let bucket = &self.current_bucket();
        let prefix = self.current_object_prefix();
        let result = client.open_management_console_list(bucket, &prefix);
        if let Err(e) = result {
            self.tx.send(AppEventType::NotifyError(e));
        }
    }

    pub fn object_detail_open_management_console(&self, name: String) {
        let (client, _) = self.unwrap_client_tx();
        let prefix = self.current_object_prefix();

        let result = client.open_management_console_object(&self.current_bucket(), &prefix, &name);
        if let Err(e) = result {
            self.tx.send(AppEventType::NotifyError(e));
        }
    }

    pub fn detail_download_object_as(
        &mut self,
        file_detail: FileDetail,
        input: String,
        version_id: Option<String>,
    ) {
        self.tx.send(AppEventType::DownloadObjectAs(
            file_detail,
            input,
            version_id,
        ));
        self.app_view_state.is_loading = true;

        let page = self.page_stack.current_page_mut().as_mut_object_detail();
        page.close_save_dialog();
    }

    pub fn preview_download_object_as(
        &mut self,
        file_detail: FileDetail,
        input: String,
        version_id: Option<String>,
    ) {
        self.tx.send(AppEventType::DownloadObjectAs(
            file_detail,
            input,
            version_id,
        ));
        self.app_view_state.is_loading = true;

        let page = self.page_stack.current_page_mut().as_mut_object_preview();
        page.close_save_dialog();
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

    pub fn warn_notification(&mut self, msg: String) {
        self.app_view_state.notification = Notification::Warn(msg);
    }

    pub fn error_notification(&mut self, e: AppError) {
        self.handle_error(&e);
        self.app_view_state.notification = Notification::Error(e.msg);
    }

    fn handle_error(&self, e: &AppError) {
        tracing::error!("AppError occurred: {:?}", e);

        // cause panic if save errors
        let path = self.config.error_log_path().unwrap();
        save_error_log(&path, e).unwrap();
    }

    pub fn dump_app(&self) {
        tracing::debug!("{:?}", self);
    }

    fn unwrap_client_tx(&self) -> (Arc<Client>, Sender) {
        (self.client.as_ref().unwrap().clone(), self.tx.clone())
    }
}
