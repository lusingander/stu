use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    widgets::Block,
    Frame,
};
use std::{
    io::{BufWriter, Write},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};
use tokio::spawn;

use crate::{
    client::Client,
    color::ColorTheme,
    config::Config,
    environment::Environment,
    error::{AppError, Result},
    event::{
        AppEventType, CompleteDownloadObjectResult, CompleteDownloadObjectsResult,
        CompleteInitializeResult, CompleteLoadAllDownloadObjectListResult,
        CompleteLoadObjectDetailResult, CompleteLoadObjectVersionsResult,
        CompleteLoadObjectsResult, CompletePreviewObjectResult, CompleteReloadBucketsResult,
        CompleteReloadObjectsResult, CompleteSaveObjectResult, Sender,
    },
    file::{copy_to_clipboard, create_binary_file, save_error_log},
    keys::UserEventMapper,
    object::{AppObjects, DownloadObjectInfo, FileDetail, ObjectItem, ObjectKey, RawObject},
    pages::page::{Page, PageStack},
    widget::{Header, LoadingDialog, Status, StatusType},
};

#[derive(Debug)]
pub enum Notification {
    None,
    Info(String),
    Success(String),
    Warn(String),
    Error(String),
}

#[derive(Debug, Default)]
pub struct AppContext {
    pub config: Config,
    pub env: Environment,
    pub theme: ColorTheme,
}

impl AppContext {
    pub fn new(config: Config, env: Environment, theme: ColorTheme) -> AppContext {
        AppContext { config, env, theme }
    }
}

#[derive(Debug)]
pub struct App<C: Client> {
    pub page_stack: PageStack,
    pub mapper: UserEventMapper,
    app_objects: AppObjects,
    client: Arc<C>,
    ctx: Rc<AppContext>,
    tx: Sender,

    notification: Notification,
    is_loading: bool,
}

impl<C: Client> App<C> {
    pub fn new(mapper: UserEventMapper, client: C, ctx: AppContext, tx: Sender) -> App<C> {
        let ctx = Rc::new(ctx);
        App {
            app_objects: AppObjects::default(),
            page_stack: PageStack::new(Rc::clone(&ctx), tx.clone()),
            mapper,
            client: Arc::new(client),
            ctx,
            tx,
            notification: Notification::None,
            is_loading: true,
        }
    }

    pub fn initialize(&mut self, bucket: Option<String>) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        spawn(async move {
            let buckets = match bucket {
                Some(name) => client.load_bucket(&name).await,
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

                if self.app_objects.get_bucket_items().len() > 1 {
                    // if multiple buckets are found, show bucket list page
                    let bucket_list_page = Page::of_bucket_list(
                        self.app_objects.get_bucket_items(),
                        Rc::clone(&self.ctx),
                        self.tx.clone(),
                    );
                    self.page_stack.push(bucket_list_page);
                }
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }

        let bucket_items = self.app_objects.get_bucket_items();

        if bucket_items.len() == 1 {
            // bucket name is specified, or if there is only one bucket, open it.
            // since continues to load object, is_loading is not reset.
            let object_key = ObjectKey::bucket(&bucket_items[0].name);
            self.bucket_list_move_down(object_key);
        } else {
            if bucket_items.is_empty() {
                let msg = format!("No bucket found (region: {})", self.client.region());
                self.tx.send(AppEventType::NotifyWarn(msg));
            }
            self.is_loading = false;
        }
    }

    pub fn reload_buckets(&self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
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

    pub fn bucket_list_move_down(&mut self, object_key: ObjectKey) {
        if let Some(current_object_items) = self.app_objects.get_object_items(&object_key) {
            // object list has been already loaded
            let object_list_page = Page::of_object_list(
                current_object_items,
                object_key,
                Rc::clone(&self.ctx),
                self.tx.clone(),
            );
            self.page_stack.push(object_list_page);
        } else {
            self.tx.send(AppEventType::LoadObjects(object_key));
            self.is_loading = true;
        }
    }

    pub fn bucket_list_refresh(&mut self) {
        self.app_objects.clear_all();

        self.tx.send(AppEventType::ReloadBuckets);
        self.is_loading = true;
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
                        Rc::clone(&self.ctx),
                        self.tx.clone(),
                    );
                    self.page_stack.push(object_detail_page);
                } else {
                    self.tx.send(AppEventType::LoadObjectDetail);
                    self.is_loading = true;
                }
            }
            ObjectItem::Dir { .. } => {
                let object_key = object_list_page.current_selected_object_key();
                if let Some(current_object_items) = self.app_objects.get_object_items(&object_key) {
                    // object list has been already loaded
                    let new_object_list_page = Page::of_object_list(
                        current_object_items,
                        object_key,
                        Rc::clone(&self.ctx),
                        self.tx.clone(),
                    );
                    self.page_stack.push(new_object_list_page);
                } else {
                    self.tx.send(AppEventType::LoadObjects(object_key));
                    self.is_loading = true;
                }
            }
        }
    }

    pub fn object_list_move_up(&mut self) {
        if self.page_stack.len() == 1 && self.app_objects.get_bucket_items().len() == 1 {
            return;
        }
        self.page_stack.pop();
    }

    pub fn object_list_refresh(&mut self) {
        let object_list_page = self.page_stack.current_page().as_object_list();
        let object_key = object_list_page.current_dir_object_key();
        self.app_objects.clear_object_items_under(object_key);

        self.tx.send(AppEventType::ReloadObjects);
        self.is_loading = true;
    }

    pub fn back_to_bucket_list(&mut self) {
        if self.app_objects.get_bucket_items().len() == 1 {
            return;
        }
        self.page_stack.clear();
    }

    pub fn load_objects(&self, current_object_key: ObjectKey) {
        let bucket = current_object_key.bucket_name.clone();
        let prefix = current_object_key.joined_object_path(false);

        let client = self.client.clone();
        let tx = self.tx.clone();
        spawn(async move {
            let items = client.load_objects(&bucket, &prefix).await;
            let result = CompleteLoadObjectsResult::new(items, current_object_key);
            tx.send(AppEventType::CompleteLoadObjects(result));
        });
    }

    pub fn complete_load_objects(&mut self, result: Result<CompleteLoadObjectsResult>) {
        match result {
            Ok(CompleteLoadObjectsResult { items, object_key }) => {
                self.app_objects
                    .set_object_items(object_key.clone(), items.clone());

                let object_list_page =
                    Page::of_object_list(items, object_key, Rc::clone(&self.ctx), self.tx.clone());
                self.page_stack.push(object_list_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.is_loading = false;
    }

    pub fn reload_objects(&self) {
        let object_list_page = self.page_stack.current_page().as_object_list();
        let object_key = object_list_page.current_dir_object_key().clone();
        let bucket = object_key.bucket_name.clone();
        let prefix = object_key.joined_object_path(false);

        let client = self.client.clone();
        let tx = self.tx.clone();
        spawn(async move {
            let items = client.load_objects(&bucket, &prefix).await;
            let result = CompleteReloadObjectsResult::new(items, object_key);
            tx.send(AppEventType::CompleteReloadObjects(result));
        });
    }

    pub fn complete_reload_objects(&mut self, result: Result<CompleteReloadObjectsResult>) {
        self.page_stack.pop();
        self.complete_load_objects(result.map(|r| r.into()));
    }

    pub fn load_object_detail(&self) {
        let object_list_page = self.page_stack.current_page().as_object_list();

        if let ObjectItem::File { name, .. } = object_list_page.current_selected_item() {
            let name = name.clone();

            let map_key = object_list_page.current_selected_object_key().clone();
            let bucket = map_key.bucket_name.clone();
            let key = map_key.joined_object_path(true);

            let client = self.client.clone();
            let tx = self.tx.clone();
            spawn(async move {
                let detail = client.load_object_detail(&bucket, &key, &name).await;
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
                    Rc::clone(&self.ctx),
                    self.tx.clone(),
                );
                self.page_stack.push(object_detail_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.is_loading = false;
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
            self.is_loading = true;
        }
    }

    pub fn load_object_versions(&self) {
        let object_detail_page = self.page_stack.current_page().as_object_detail();

        let map_key = object_detail_page.current_object_key().clone();
        let bucket = map_key.bucket_name.clone();
        let key = map_key.joined_object_path(true);

        let client = self.client.clone();
        let tx = self.tx.clone();
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
        self.is_loading = false;
    }

    pub fn start_load_all_download_objects(&mut self, key: ObjectKey, download_as: bool) {
        self.tx
            .send(AppEventType::LoadAllDownloadObjectList(key, download_as));
        self.is_loading = true;
    }

    pub fn load_all_download_objects(&self, key: ObjectKey, download_as: bool) {
        let bucket = key.bucket_name.clone();
        let prefix = key.joined_object_path(false);

        let client = self.client.clone();
        let tx = self.tx.clone();
        spawn(async move {
            let objects = client.list_all_download_objects(&bucket, &prefix).await;
            let result = CompleteLoadAllDownloadObjectListResult::new(objects, download_as);
            tx.send(AppEventType::CompleteLoadAllDownloadObjectList(result));
        });
    }

    pub fn complete_load_all_download_objects(
        &mut self,
        result: Result<CompleteLoadAllDownloadObjectListResult>,
    ) {
        match result {
            Ok(CompleteLoadAllDownloadObjectListResult { objs, download_as }) => {
                match self.page_stack.current_page_mut() {
                    Page::BucketList(page) => {
                        page.open_download_confirm_dialog(objs, download_as);
                    }
                    Page::ObjectList(page) => {
                        page.open_download_confirm_dialog(objs, download_as);
                    }
                    page => panic!("Invalid page: {:?}", page),
                }
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.is_loading = false;
    }

    pub fn open_help(&mut self) {
        let helps = self.page_stack.current_page().helps(&self.mapper);
        if helps.is_empty() {
            return;
        }
        let help_page = Page::of_help(helps, Rc::clone(&self.ctx), self.tx.clone());
        self.page_stack.push(help_page);
    }

    pub fn close_current_page(&mut self) {
        self.page_stack.pop();
    }

    pub fn open_preview(
        &mut self,
        object_key: ObjectKey,
        file_detail: FileDetail,
        version_id: Option<String>,
    ) {
        self.tx.send(AppEventType::PreviewObject(
            object_key,
            file_detail,
            version_id,
        ));
        self.is_loading = true;
    }

    pub fn start_download_object(
        &mut self,
        object_key: ObjectKey,
        object_name: String,
        size_byte: usize,
        version_id: Option<String>,
    ) {
        self.tx.send(AppEventType::DownloadObject(
            object_key,
            object_name,
            size_byte,
            version_id,
        ));
        self.is_loading = true;
    }

    pub fn download_object(
        &self,
        object_key: ObjectKey,
        object_name: String,
        size_byte: usize,
        version_id: Option<String>,
    ) {
        let bucket = object_key.bucket_name.clone();
        let key = object_key.joined_object_path(true);

        let path = self.ctx.config.download_file_path(&object_name);
        let writer = create_binary_file(&path);

        let client = self.client.clone();
        let tx = self.tx.clone();
        let loading = self.handle_loading_size(size_byte, tx.clone());

        spawn(async move {
            match writer {
                Ok(mut writer) => {
                    let result = client
                        .download_object(&bucket, &key, version_id, &mut writer, loading)
                        .await;
                    let result = CompleteDownloadObjectResult::new(result, path);
                    tx.send(AppEventType::CompleteDownloadObject(result));
                }
                Err(e) => {
                    tx.send(AppEventType::CompleteDownloadObject(Err(e)));
                }
            }
        });
    }

    pub fn start_download_object_as(
        &mut self,
        object_key: ObjectKey,
        size_byte: usize,
        input: String,
        version_id: Option<String>,
    ) {
        self.tx.send(AppEventType::DownloadObjectAs(
            object_key, size_byte, input, version_id,
        ));
        self.is_loading = true;
    }

    pub fn download_object_as(
        &self,
        object_key: ObjectKey,
        size_byte: usize,
        input: String,
        version_id: Option<String>,
    ) {
        let bucket = object_key.bucket_name.clone();
        let key = object_key.joined_object_path(true);

        let path = self.ctx.config.download_file_path(&input);
        let writer = create_binary_file(&path);

        let client = self.client.clone();
        let tx = self.tx.clone();
        let loading = self.handle_loading_size(size_byte, tx.clone());

        spawn(async move {
            match writer {
                Ok(mut writer) => {
                    let result = client
                        .download_object(&bucket, &key, version_id, &mut writer, loading)
                        .await;
                    let result = CompleteDownloadObjectResult::new(result, path);
                    tx.send(AppEventType::CompleteDownloadObject(result));
                }
                Err(e) => {
                    tx.send(AppEventType::CompleteDownloadObject(Err(e)));
                }
            }
        });
    }

    pub fn complete_download_object(&mut self, result: Result<CompleteDownloadObjectResult>) {
        match result {
            Ok(CompleteDownloadObjectResult { path }) => {
                let msg = format!(
                    "Download completed successfully: {}",
                    path.to_string_lossy()
                );
                self.tx.send(AppEventType::NotifySuccess(msg));
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.is_loading = false;
    }

    pub fn download_objects(
        &mut self,
        bucket: String,
        key: ObjectKey,
        dir: String,
        objs: Vec<DownloadObjectInfo>,
    ) {
        self.is_loading = true;

        let current_selected_dir_key = key.joined_object_path(false);
        let mut obj_paths = Vec::with_capacity(objs.len());
        for obj in objs {
            let relative_path =
                PathBuf::from(&dir).join(obj.key.strip_prefix(&current_selected_dir_key).unwrap());
            let absolute_path = self.ctx.config.download_file_path(relative_path);
            obj_paths.push((obj, absolute_path));
        }
        let download_dir = self.ctx.config.download_file_path(&dir);

        let total_count = obj_paths.len();
        let total_size: usize = obj_paths.iter().map(|(obj, _)| obj.size_byte).sum();
        let decimal_places = if total_size > 1_000_000_000 { 1 } else { 0 };
        let format_opt =
            humansize::FormatSizeOptions::from(humansize::DECIMAL).decimal_places(decimal_places);
        let total_size_s = humansize::format_size_i(total_size, format_opt);

        let max_concurrent_requests = self.ctx.config.max_concurrent_requests;

        let client = self.client.clone();
        let tx = self.tx.clone();

        spawn(async move {
            let mut iter = futures::stream::iter(obj_paths)
                .map(|(obj, path)| {
                    let bucket = bucket.clone();
                    let client = client.clone();
                    async move {
                        let mut writer = create_binary_file(path)?;
                        client
                            .download_object(&bucket, &obj.key, None, &mut writer, |_| {})
                            .await?;
                        Ok(obj.size_byte)
                    }
                })
                .buffered(max_concurrent_requests);

            let mut cur_count = 0;
            let mut cur_size = 0;
            while let Some(result) = iter.next().await {
                match result {
                    Ok(size) => {
                        cur_count += 1;
                        cur_size += size;

                        let cur_size_s = humansize::format_size_i(cur_size, format_opt);
                        let msg = format!(
                            "{}/{} objects downloaded ({} out of {} total)",
                            cur_count, total_count, cur_size_s, total_size_s
                        );
                        tx.send(AppEventType::NotifyInfo(msg));
                    }
                    Err(e) => {
                        tx.send(AppEventType::CompleteDownloadObjects(Err(e)));
                        return;
                    }
                }
            }

            let result = CompleteDownloadObjectsResult::new(download_dir);
            tx.send(AppEventType::CompleteDownloadObjects(result));
        });
    }

    pub fn complete_download_objects(&mut self, result: Result<CompleteDownloadObjectsResult>) {
        match result {
            Ok(CompleteDownloadObjectsResult { download_dir }) => {
                let msg = format!(
                    "Download completed successfully: {}",
                    download_dir.to_string_lossy()
                );
                self.tx.send(AppEventType::NotifySuccess(msg));
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.is_loading = false;
    }

    pub fn preview_object(
        &self,
        object_key: ObjectKey,
        file_detail: FileDetail,
        version_id: Option<String>,
    ) {
        let size_byte = file_detail.size_byte;

        let bucket = object_key.bucket_name.clone();
        let key = object_key.joined_object_path(true);

        let client = self.client.clone();
        let tx = self.tx.clone();
        let loading = self.handle_loading_size(size_byte, tx.clone());

        spawn(async move {
            let mut bytes = Vec::with_capacity(size_byte);
            let result = {
                let mut writer = BufWriter::new(&mut bytes);
                client
                    .download_object(&bucket, &key, version_id.clone(), &mut writer, loading)
                    .await
            };
            let obj = result.map(|_| RawObject { bytes });
            let result = CompletePreviewObjectResult::new(obj, file_detail, version_id);
            tx.send(AppEventType::CompletePreviewObject(result));
        });
    }

    pub fn complete_preview_object(&mut self, result: Result<CompletePreviewObjectResult>) {
        match result {
            Ok(CompletePreviewObjectResult {
                obj,
                file_detail,
                file_version_id,
            }) => {
                let object_preview_page = Page::of_object_preview(
                    file_detail,
                    file_version_id,
                    obj,
                    Rc::clone(&self.ctx),
                    self.tx.clone(),
                );
                self.page_stack.push(object_preview_page);
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        };
        self.clear_notification();
        self.is_loading = false;
    }

    pub fn start_save_object(&mut self, name: String, obj: Arc<RawObject>) {
        self.tx.send(AppEventType::SaveObject(name, obj));
        self.is_loading = true;
    }

    pub fn save_object(&self, name: String, obj: Arc<RawObject>) {
        let path = self.ctx.config.download_file_path(&name);
        let writer = create_binary_file(&path);

        let tx = self.tx.clone();
        spawn(async move {
            match writer {
                Ok(mut writer) => {
                    let result = writer.write_all(&obj.bytes).map_err(AppError::error);
                    let result = CompleteSaveObjectResult::new(result, path);
                    tx.send(AppEventType::CompleteSaveObject(result));
                }
                Err(e) => {
                    tx.send(AppEventType::CompleteSaveObject(Err(e)));
                }
            }
        });
    }

    pub fn complete_save_object(&mut self, result: Result<CompleteSaveObjectResult>) {
        match result {
            Ok(CompleteSaveObjectResult { path }) => {
                let msg = format!(
                    "Download completed successfully: {}",
                    path.to_string_lossy()
                );
                self.tx.send(AppEventType::NotifySuccess(msg));
            }
            Err(e) => {
                self.tx.send(AppEventType::NotifyError(e));
            }
        }
        self.is_loading = false;

        if let Page::ObjectPreview(page) = self.page_stack.current_page() {
            if page.is_image_preview() {
                self.tx.send(AppEventType::PreviewRerenderImage);
            }
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
        let result = self.client.open_management_console_buckets();
        if let Err(e) = result {
            self.tx.send(AppEventType::NotifyError(e));
        }
    }

    pub fn object_list_open_management_console(&self, object_key: ObjectKey) {
        let bucket = &object_key.bucket_name;
        let prefix = &object_key.joined_object_path(false);
        let result = self.client.open_management_console_list(bucket, prefix);
        if let Err(e) = result {
            self.tx.send(AppEventType::NotifyError(e));
        }
    }

    pub fn object_detail_open_management_console(&self, object_key: ObjectKey) {
        let bucket = &object_key.bucket_name;
        let prefix = &object_key.joined_object_path(true); // should contains file name
        let result = self.client.open_management_console_object(bucket, prefix);
        if let Err(e) = result {
            self.tx.send(AppEventType::NotifyError(e));
        }
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

    pub fn loading(&self) -> bool {
        self.is_loading
    }

    pub fn current_notification(&self) -> &Notification {
        &self.notification
    }

    pub fn is_showing_notification(&self) -> bool {
        !matches!(self.notification, Notification::None)
    }

    pub fn clear_notification(&mut self) {
        self.notification = Notification::None;
    }

    pub fn info_notification(&mut self, msg: String) {
        self.notification = Notification::Info(msg);
    }

    pub fn success_notification(&mut self, msg: String) {
        self.notification = Notification::Success(msg);
    }

    pub fn warn_notification(&mut self, msg: String) {
        self.notification = Notification::Warn(msg);
    }

    pub fn error_notification(&mut self, e: AppError) {
        self.handle_error(&e);
        self.notification = Notification::Error(e.msg);
    }

    fn handle_error(&self, e: &AppError) {
        tracing::error!("AppError occurred: {:?}", e);

        // cause panic if save errors
        let path = self.ctx.config.error_log_path().unwrap();
        save_error_log(path, e).unwrap();
    }

    pub fn dump_app(&self) {
        tracing::debug!("{:?}", self);
    }
}

impl<C: Client> App<C> {
    pub fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Length(self.header_height()),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(f.area());

        self.render_background(f, f.area());
        self.render_header(f, chunks[0]);
        self.render_content(f, chunks[1]);
        self.render_footer(f, chunks[2]);
        self.render_loading_dialog(f);
    }

    fn header_height(&self) -> u16 {
        match self.page_stack.current_page() {
            Page::Help(_) => 0, // Hide header
            _ => 3,
        }
    }

    fn render_background(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().bg(self.ctx.theme.bg);
        f.render_widget(block, area);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        if !area.is_empty() {
            let header = Header::new(self.breadcrumb()).theme(&self.ctx.theme);
            f.render_widget(header, area);
        }
    }

    fn render_content(&mut self, f: &mut Frame, area: Rect) {
        self.page_stack.current_page_mut().render(f, area);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let status_type = match self.current_notification() {
            Notification::Info(msg) => StatusType::Info(msg.into()),
            Notification::Success(msg) => StatusType::Success(msg.into()),
            Notification::Warn(msg) => StatusType::Warn(msg.into()),
            Notification::Error(msg) => StatusType::Error(msg.into()),
            Notification::None => {
                StatusType::Help(self.page_stack.current_page().short_helps(&self.mapper))
            }
        };
        let status = Status::new(status_type).theme(&self.ctx.theme);
        f.render_widget(status, area);
    }

    fn render_loading_dialog(&self, f: &mut Frame) {
        if self.loading() {
            let dialog = LoadingDialog::default().theme(&self.ctx.theme);
            f.render_widget(dialog, f.area());
        }
    }

    fn breadcrumb(&self) -> Vec<String> {
        self.page_stack
            .iter()
            .filter_map(|page| match page {
                Page::ObjectList(page) => Some(page),
                _ => None,
            })
            .next_back()
            .map(|page| page.current_dir_object_key().paths())
            .unwrap_or_default()
    }
}
