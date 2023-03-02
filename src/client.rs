use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{output::ListObjectsV2Output, Region};
use chrono::TimeZone;

use crate::app::{FileDetail, FileVersion, Item};

const DELIMITER: &str = "/";
const DEFAULT_REGION: &str = "ap-northeast-1";

pub struct Client {
    pub client: aws_sdk_s3::Client,
    region: String,
}

impl Client {
    pub async fn new(
        region: Option<String>,
        endpoint_url: Option<String>,
        profile: Option<String>,
    ) -> Client {
        let region_provider = RegionProviderChain::first_try(region.map(Region::new))
            .or_default_provider()
            .or_else(DEFAULT_REGION);

        let mut config_loader = aws_config::from_env().region(region_provider);
        if let Some(url) = &endpoint_url {
            config_loader = config_loader.endpoint_url(url);
        }
        if let Some(profile) = &profile {
            config_loader = config_loader.profile_name(profile);
        }
        let sdk_config = config_loader.load().await;

        let mut config_builder = aws_sdk_s3::config::Builder::from(&sdk_config);
        if endpoint_url.is_some() {
            config_builder = config_builder.force_path_style(true);
        }
        let config = config_builder.build();

        let client = aws_sdk_s3::Client::from_conf(config);
        let region = sdk_config.region().unwrap().to_string();

        Client { client, region }
    }

    pub async fn load_all_buckets(&self) -> Result<Vec<Item>, String> {
        let result = self.client.list_buckets().send().await;
        let output = result.map_err(|_| "Failed to load bucket".to_string())?;

        let buckets: Vec<Item> = output
            .buckets()
            .unwrap_or_default()
            .iter()
            .map(|bucket| {
                let name = bucket.name().unwrap().to_string();
                Item::Bucket { name }
            })
            .collect();

        if buckets.is_empty() {
            Err("No buckets exist".to_string())
        } else {
            Ok(buckets)
        }
    }

    pub async fn load_objects(
        &self,
        bucket: &String,
        prefix: &String,
    ) -> Result<Vec<Item>, String> {
        let mut dirs_vec: Vec<Vec<Item>> = Vec::new();
        let mut files_vec: Vec<Vec<Item>> = Vec::new();

        let mut token: Option<String> = None;
        loop {
            let result = self
                .client
                .list_objects_v2()
                .bucket(bucket)
                .prefix(prefix)
                .delimiter(DELIMITER)
                .set_continuation_token(token)
                .send()
                .await;
            let output = result.map_err(|_| "Failed to load objects".to_string())?;

            let dirs = objects_output_to_dirs(&output);
            dirs_vec.push(dirs);

            let files = objects_output_to_files(&output);
            files_vec.push(files);

            token = output.next_continuation_token().map(|s| s.to_string());
            if token.is_none() {
                break;
            }
        }

        let di = dirs_vec.into_iter().flatten();
        let fi = files_vec.into_iter().flatten();
        Ok(di.chain(fi).collect())
    }

    pub async fn load_object_detail(
        &self,
        bucket: &String,
        key: &String,
        name: &String,
        size_byte: i64,
    ) -> Result<FileDetail, String> {
        let result = self
            .client
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await;
        let output = result.map_err(|_| "Failed to load object detail".to_string())?;

        let name = name.to_owned();
        let last_modified = convert_datetime(output.last_modified().unwrap());
        let e_tag = output.e_tag().unwrap().trim_matches('"').to_string();
        let content_type = output.content_type().unwrap().to_string();
        Ok(FileDetail {
            name,
            size_byte,
            last_modified,
            e_tag,
            content_type,
        })
    }

    pub async fn load_object_versions(
        &self,
        bucket: &String,
        key: &String,
    ) -> Result<Vec<FileVersion>, String> {
        let result = self
            .client
            .list_object_versions()
            .bucket(bucket)
            .prefix(key)
            .send()
            .await;
        let output = result.map_err(|_| "Failed to load object versions".to_string())?;

        let versions = output
            .versions()
            .unwrap_or_default()
            .iter()
            .map(|v| {
                let version_id = v.version_id().unwrap().to_string(); // returns "null" if empty...
                let size_byte = v.size();
                let last_modified = convert_datetime(v.last_modified().unwrap());
                let is_latest = v.is_latest();
                FileVersion {
                    version_id,
                    size_byte,
                    last_modified,
                    is_latest,
                }
            })
            .collect();
        Ok(versions)
    }

    pub fn open_management_console_buckets(&self) -> Result<(), String> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/buckets?region={}",
            self.region
        );
        open::that(path).map_err(|e| e.to_string())
    }

    pub fn open_management_console_list(
        &self,
        bucket: &String,
        prefix: &String,
    ) -> Result<(), String> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/buckets/{}?region={}&prefix={}",
            bucket, self.region, prefix
        );
        open::that(path).map_err(|e| e.to_string())
    }

    pub fn open_management_console_object(
        &self,
        bucket: &String,
        prefix: &String,
        name: &String,
    ) -> Result<(), String> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/object/{}?region={}&prefix={}{}",
            bucket, self.region, prefix, name
        );
        open::that(path).map_err(|e| e.to_string())
    }
}

fn objects_output_to_dirs(output: &ListObjectsV2Output) -> Vec<Item> {
    let objects = output.common_prefixes().unwrap_or_default();
    objects
        .iter()
        .map(|dir| {
            let path = dir.prefix().unwrap().to_string();
            let paths = parse_path(&path, true);
            let name = paths.last().unwrap().to_owned();
            Item::Dir { name, paths }
        })
        .collect()
}

fn objects_output_to_files(output: &ListObjectsV2Output) -> Vec<Item> {
    let objects = output.contents().unwrap_or_default();
    objects
        .iter()
        .map(|file| {
            let path = file.key().unwrap().to_string();
            let paths = parse_path(&path, false);
            let name = paths.last().unwrap().to_owned();
            let size_byte = file.size();
            let last_modified = convert_datetime(file.last_modified().unwrap());
            Item::File {
                name,
                paths,
                size_byte,
                last_modified,
            }
        })
        .collect()
}

fn parse_path(path: &str, dir: bool) -> Vec<String> {
    let ss: Vec<String> = path.split(DELIMITER).map(|s| s.to_string()).collect();
    if dir {
        let n = ss.len() - 1;
        ss.into_iter().take(n).collect()
    } else {
        ss
    }
}

fn convert_datetime(dt: &aws_smithy_types::DateTime) -> chrono::DateTime<chrono::Local> {
    let nanos = dt.as_nanos();
    chrono::Local.timestamp_nanos(nanos as i64)
}
