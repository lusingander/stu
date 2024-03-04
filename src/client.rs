use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{config::Region, operation::list_objects_v2::ListObjectsV2Output};
use chrono::TimeZone;

use crate::{
    error::{AppError, Result},
    item::{BucketItem, FileDetail, FileVersion, Object, ObjectItem},
};

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

        let mut config_loader =
            aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
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

    pub async fn load_all_buckets(&self) -> Result<Vec<BucketItem>> {
        let result = self.client.list_buckets().send().await;
        let output = result.map_err(|e| AppError::new("Failed to load bucket", e))?;

        let buckets: Vec<BucketItem> = output
            .buckets()
            .iter()
            .map(|bucket| {
                let name = bucket.name().unwrap().to_string();
                BucketItem { name }
            })
            .collect();

        if buckets.is_empty() {
            Err(AppError::msg("No buckets exist"))
        } else {
            Ok(buckets)
        }
    }

    pub async fn load_objects(&self, bucket: &String, prefix: &String) -> Result<Vec<ObjectItem>> {
        let mut dirs_vec: Vec<Vec<ObjectItem>> = Vec::new();
        let mut files_vec: Vec<Vec<ObjectItem>> = Vec::new();

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
            let output = result.map_err(|e| AppError::new("Failed to load objects", e))?;

            let dirs = objects_output_to_dirs(&output);
            dirs_vec.push(dirs);

            let files = objects_output_to_files(&output);
            files_vec.push(files);

            token = output.next_continuation_token().map(String::from);
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
    ) -> Result<FileDetail> {
        let result = self
            .client
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await;
        let output = result.map_err(|e| AppError::new("Failed to load object detail", e))?;

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
    ) -> Result<Vec<FileVersion>> {
        let result = self
            .client
            .list_object_versions()
            .bucket(bucket)
            .prefix(key)
            .send()
            .await;
        let output = result.map_err(|e| AppError::new("Failed to load object versions", e))?;

        let versions = output
            .versions()
            .iter()
            .map(|v| {
                let version_id = v.version_id().unwrap().to_string(); // returns "null" if empty...
                let size_byte = v.size().unwrap();
                let last_modified = convert_datetime(v.last_modified().unwrap());
                let is_latest = v.is_latest().unwrap();
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

    pub async fn download_object(&self, bucket: &String, key: &String) -> Result<Object> {
        let result = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await;
        let output = result.map_err(|e| AppError::new("Failed to download object", e))?;

        // todo: stream
        let data = output
            .body
            .collect()
            .await
            .map_err(|e| AppError::new("Failed to collect body", e))?;
        let content_type = output.content_type.unwrap();

        Ok(Object {
            content_type,
            bytes: data.to_vec(),
        })
    }

    pub fn open_management_console_buckets(&self) -> Result<()> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/buckets?region={}",
            self.region
        );
        open::that(path).map_err(AppError::error)
    }

    pub fn open_management_console_list(&self, bucket: &String, prefix: &String) -> Result<()> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/buckets/{}?region={}&prefix={}",
            bucket, self.region, prefix
        );
        open::that(path).map_err(AppError::error)
    }

    pub fn open_management_console_object(
        &self,
        bucket: &String,
        prefix: &String,
        name: &String,
    ) -> Result<()> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/object/{}?region={}&prefix={}{}",
            bucket, self.region, prefix, name
        );
        open::that(path).map_err(AppError::error)
    }
}

fn objects_output_to_dirs(output: &ListObjectsV2Output) -> Vec<ObjectItem> {
    let objects = output.common_prefixes();
    objects
        .iter()
        .map(|dir| {
            let path = dir.prefix().unwrap();
            let paths = parse_path(path, true);
            let name = paths.last().unwrap().to_owned();
            ObjectItem::Dir { name, paths }
        })
        .collect()
}

fn objects_output_to_files(output: &ListObjectsV2Output) -> Vec<ObjectItem> {
    let objects = output.contents();
    objects
        .iter()
        .map(|file| {
            let path = file.key().unwrap();
            let paths = parse_path(path, false);
            let name = paths.last().unwrap().to_owned();
            let size_byte = file.size().unwrap();
            let last_modified = convert_datetime(file.last_modified().unwrap());
            ObjectItem::File {
                name,
                paths,
                size_byte,
                last_modified,
            }
        })
        .collect()
}

fn parse_path(path: &str, dir: bool) -> Vec<String> {
    let ss: Vec<String> = path.split(DELIMITER).map(String::from).collect();
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
