use std::sync::Arc;
use tokio::spawn;

use crate::{
    client::Client,
    color::ColorTheme,
    config::Config,
    error::{AppError, Result},
    event::{
        AppEventType, CompleteDownloadObjectResult, CompleteInitializeResult,
        CompleteLoadObjectDetailResult, CompleteLoadObjectVersionsResult,
        CompleteLoadObjectsResult, CompletePreviewObjectResult, CompleteReloadBucketsResult,
        CompleteReloadObjectsResult, Sender,
    },
    file::{copy_to_clipboard, save_binary, save_error_log},
    object::{AppObjects, FileDetail, ObjectItem, RawObject},
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
    pub theme: ColorTheme,
    tx: Sender,
}

impl App {
    pub fn new(config: Config, theme: ColorTheme, tx: Sender, width: usize, height: usize) -> App {
        App {
            app_view_state: AppViewState::new(width, height),
            app_objects: AppObjects::default(),
            page_stack: PageStack::new(theme.clone(), tx.clone()),
            client: None,
            config,
            theme,
            tx,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.app_view_state.reset_size(width, height);
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

                let bucket_list_page = Page::of_bucket_list(
                    self.app_objects.get_bucket_items(),
                    self.theme.clone(),
                    self.tx.clone(),
                );
                self.page_stack.pop(); // remove initializing page
                self.page_stack.push(bucket_list_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }

        let bucket_items_len = self.app_objects.get_bucket_items().len();

        if bucket_items_len == 1 {
            // bucket name is specified, or if there is only one bucket, open it.
            // since continues to load object, is_loading is not reset.
            self.bucket_list_move_down();
        } else {
            if bucket_items_len == 0 {
                let (client, _) = self.unwrap_client_tx();
                let msg = format!("No bucket found (region: {})", client.region());
                self.tx.send(AppEventType::NotifyWarn(msg));
            }
            self.app_view_state.is_loading = false;
        }
    }

    pub fn reload_buckets(&self) {
        let (client, tx) = self.unwrap_client_tx();
        spawn(async move {
            let buckets = client.load_all_buckets().await;
            let result = CompleteReloadBucketsResult::new(buckets);
            tx.send(AppEventType::CompleteReloadBuckets(result));
        });
    }

    pub fn complete_reload_buckets(&mut self, result: Result<CompleteReloadBucketsResult>) {
        // current bucket list page is popped inside complete_initialize
        self.complete_initialize(result.map(|r| r.into()));
    }

    pub fn bucket_list_move_down(&mut self) {
        let bucket_page = self.page_stack.current_page().as_bucket_list();
        let object_key = bucket_page.current_selected_object_key();

        if let Some(current_object_items) = self.app_objects.get_object_items(&object_key) {
            // object list has been already loaded
            let object_list_page = Page::of_object_list(
                current_object_items,
                object_key,
                self.config.ui.clone(),
                self.theme.clone(),
                self.tx.clone(),
            );
            self.page_stack.push(object_list_page);
        } else {
            self.tx.send(AppEventType::LoadObjects);
            self.app_view_state.is_loading = true;
        }
    }

    pub fn bucket_list_refresh(&mut self) {
        self.app_objects.clear_all();

        self.tx.send(AppEventType::ReloadBuckets);
        self.app_view_state.is_loading = true;
    }

    pub fn object_list_move_down(&mut self) {
        let object_list_page = self.page_stack.current_page().as_object_list();
        let selected = object_list_page.current_selected_item().to_owned();

        match selected {
            ObjectItem::File { .. } => {
                let current_object_key = object_list_page.current_selected_object_key();
                let detail = self.app_objects.get_object_detail(&current_object_key);

                if let Some(detail) = detail {
                    // object detail has been already loaded
                    let object_detail_page = Page::of_object_detail(
                        detail.clone(),
                        object_list_page.object_list(),
                        current_object_key,
                        object_list_page.list_state(),
                        self.config.ui.clone(),
                        self.theme.clone(),
                        self.tx.clone(),
                    );
                    self.page_stack.push(object_detail_page);
                } else {
                    self.tx.send(AppEventType::LoadObjectDetail);
                    self.app_view_state.is_loading = true;
                }
            }
            ObjectItem::Dir { .. } => {
                let object_key = object_list_page.current_selected_object_key();
                if let Some(current_object_items) = self.app_objects.get_object_items(&object_key) {
                    // object list has been already loaded
                    let new_object_list_page = Page::of_object_list(
                        current_object_items,
                        object_key,
                        self.config.ui.clone(),
                        self.theme.clone(),
                        self.tx.clone(),
                    );
                    self.page_stack.push(new_object_list_page);
                } else {
                    self.tx.send(AppEventType::LoadObjects);
                    self.app_view_state.is_loading = true;
                }
            }
        }
    }

    pub fn object_list_move_up(&mut self) {
        if self.page_stack.len() == 2 /* bucket list and object list */ && self.app_objects.get_bucket_items().len() == 1
        {
            return;
        }
        self.page_stack.pop();
    }

    pub fn object_list_refresh(&mut self) {
        let object_list_page = self.page_stack.current_page().as_object_list();
        let object_key = object_list_page.current_dir_object_key();
        self.app_objects.clear_object_items_under(object_key);

        self.tx.send(AppEventType::ReloadObjects);
        self.app_view_state.is_loading = true;
    }

    pub fn back_to_bucket_list(&mut self) {
        if self.app_objects.get_bucket_items().len() == 1 {
            return;
        }
        self.page_stack.clear();
    }

    pub fn load_objects(&self) {
        let current_object_key = match self.page_stack.current_page() {
            page @ Page::BucketList(_) => page.as_bucket_list().current_selected_object_key(),
            page @ Page::ObjectList(_) => page.as_object_list().current_selected_object_key(),
            page => panic!("Invalid page: {:?}", page),
        };
        let bucket = current_object_key.bucket_name.clone();
        let prefix = current_object_key.joined_object_path(false);
        let (client, tx) = self.unwrap_client_tx();
        spawn(async move {
            let items = client.load_objects(&bucket, &prefix).await;
            let result = CompleteLoadObjectsResult::new(items);
            tx.send(AppEventType::CompleteLoadObjects(result));
        });
    }

    pub fn complete_load_objects(&mut self, result: Result<CompleteLoadObjectsResult>) {
        let current_object_key = match self.page_stack.current_page() {
            page @ Page::BucketList(_) => page.as_bucket_list().current_selected_object_key(),
            page @ Page::ObjectList(_) => page.as_object_list().current_selected_object_key(),
            page => panic!("Invalid page: {:?}", page),
        };

        match result {
            Ok(CompleteLoadObjectsResult { items }) => {
                self.app_objects
                    .set_object_items(current_object_key.clone(), items.clone());

                let object_list_page = Page::of_object_list(
                    items,
                    current_object_key,
                    self.config.ui.clone(),
                    self.theme.clone(),
                    self.tx.clone(),
                );
                self.page_stack.push(object_list_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn reload_objects(&self) {
        let object_list_page = self.page_stack.current_page().as_object_list();
        let object_key = object_list_page.current_dir_object_key();
        let bucket = object_key.bucket_name.clone();
        let prefix = object_key.joined_object_path(false);
        let (client, tx) = self.unwrap_client_tx();
        spawn(async move {
            let items = client.load_objects(&bucket, &prefix).await;
            let result = CompleteReloadObjectsResult::new(items);
            tx.send(AppEventType::CompleteReloadObjects(result));
        });
    }

    pub fn complete_reload_objects(&mut self, result: Result<CompleteReloadObjectsResult>) {
        self.page_stack.pop();
        self.complete_load_objects(result.map(|r| r.into()));
    }

    pub fn load_object_detail(&self) {
        let object_list_page = self.page_stack.current_page().as_object_list();

        if let ObjectItem::File {
            name, size_byte, ..
        } = object_list_page.current_selected_item()
        {
            let name = name.clone();
            let size_byte = *size_byte;

            let map_key = object_list_page.current_selected_object_key().clone();
            let bucket = map_key.bucket_name.clone();
            let key = map_key.joined_object_path(true);

            let (client, tx) = self.unwrap_client_tx();
            spawn(async move {
                let detail = client
                    .load_object_detail(&bucket, &key, &name, size_byte)
                    .await;
                let result = CompleteLoadObjectDetailResult::new(detail, map_key);
                tx.send(AppEventType::CompleteLoadObjectDetail(result));
            });
        }
    }

    pub fn complete_load_object_detail(&mut self, result: Result<CompleteLoadObjectDetailResult>) {
        match result {
            Ok(CompleteLoadObjectDetailResult { detail, map_key }) => {
                self.app_objects
                    .set_object_detail(map_key.clone(), *detail.clone());

                let object_page = self.page_stack.current_page().as_object_list();

                let object_detail_page = Page::of_object_detail(
                    *detail.clone(),
                    object_page.object_list(),
                    map_key,
                    object_page.list_state(),
                    self.config.ui.clone(),
                    self.theme.clone(),
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

    pub fn open_object_versions_tab(&mut self) {
        let object_detail_page = self.page_stack.current_page().as_object_detail();

        let current_object_key = object_detail_page.current_object_key().clone();
        let versions = self.app_objects.get_object_versions(&current_object_key);

        if let Some(versions) = versions {
            // object versions has been already loaded
            let result =
                CompleteLoadObjectVersionsResult::new(Ok(versions.clone()), current_object_key);
            self.tx
                .send(AppEventType::CompleteLoadObjectVersions(result));
        } else {
            self.tx.send(AppEventType::LoadObjectVersions);
            self.app_view_state.is_loading = true;
        }
    }

    pub fn load_object_versions(&self) {
        let object_detail_page = self.page_stack.current_page().as_object_detail();

        let map_key = object_detail_page.current_object_key().clone();
        let bucket = map_key.bucket_name.clone();
        let key = map_key.joined_object_path(true);

        let (client, tx) = self.unwrap_client_tx();
        spawn(async move {
            let versions = client.load_object_versions(&bucket, &key).await;
            let result = CompleteLoadObjectVersionsResult::new(versions, map_key);
            tx.send(AppEventType::CompleteLoadObjectVersions(result));
        });
    }

    pub fn complete_load_object_versions(
        &mut self,
        result: Result<CompleteLoadObjectVersionsResult>,
    ) {
        match result {
            Ok(CompleteLoadObjectVersionsResult { versions, map_key }) => {
                self.app_objects
                    .set_object_versions(map_key, versions.clone());

                let object_detail_page = self.page_stack.current_page_mut().as_mut_object_detail();
                object_detail_page.set_versions(versions);
                object_detail_page.select_versions_tab();
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.app_view_state.is_loading = false;
    }

    pub fn open_help(&mut self) {
        let helps = self.page_stack.current_page().helps();
        if helps.is_empty() {
            return;
        }
        let help_page = Page::of_help(helps, self.theme.clone(), self.tx.clone());
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

        if let Page::ObjectPreview(page) = self.page_stack.current_page() {
            if page.is_image_preview() {
                self.tx.send(AppEventType::PreviewRerenderImage);
            }
        }
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
        let object_detail_page = self.page_stack.current_page().as_object_detail();
        let current_object_key = object_detail_page.current_object_key().clone();

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
                    current_object_key,
                    self.config.preview.clone(),
                    self.theme.clone(),
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
        let object_key = match self.page_stack.current_page() {
            page @ Page::ObjectDetail(_) => page.as_object_detail().current_object_key(),
            page @ Page::ObjectPreview(_) => page.as_object_preview().current_object_key(),
            page => panic!("Invalid page: {:?}", page),
        };

        let bucket = object_key.bucket_name.clone();
        let key = object_key.joined_object_path(true);

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
        let object_list_page = self.page_stack.current_page().as_object_list();
        let object_key = object_list_page.current_dir_object_key();

        let (client, _) = self.unwrap_client_tx();
        let bucket = &object_key.bucket_name;
        let prefix = &object_key.joined_object_path(false);
        let result = client.open_management_console_list(bucket, prefix);
        if let Err(e) = result {
            self.tx.send(AppEventType::NotifyError(e));
        }
    }

    pub fn object_detail_open_management_console(&self) {
        let object_detail_page = self.page_stack.current_page().as_object_detail();
        let object_key = object_detail_page.current_object_key();

        let (client, _) = self.unwrap_client_tx();
        let bucket = &object_key.bucket_name;
        let prefix = &object_key.joined_object_path(true); // should contains file name
        let result = client.open_management_console_object(bucket, prefix);
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

    pub fn preview_rerender_image(&mut self) {
        let object_preview_page = self.page_stack.current_page_mut().as_mut_object_preview();
        object_preview_page.enable_image_render();
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
