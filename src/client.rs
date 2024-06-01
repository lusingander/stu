use std::fmt::Debug;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{config::Region, operation::list_objects_v2::ListObjectsV2Output};
use chrono::TimeZone;

use crate::{
    error::{AppError, Result},
    object::{BucketItem, FileDetail, FileVersion, ObjectItem, RawObject},
};

const DELIMITER: &str = "/";
const DEFAULT_REGION: &str = "ap-northeast-1";

pub struct Client {
    pub client: aws_sdk_s3::Client,
    region: String,
}

impl Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client {{ region: {} }}", self.region)
    }
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
        let output = result.map_err(|e| AppError::new("Failed to load buckets", e))?;

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

    pub async fn load_bucket(&self, name: &str) -> Result<BucketItem> {
        let result = self.client.head_bucket().bucket(name).send().await;
        // Check only existence and accessibility
        result.map_err(|e| AppError::new(format!("Failed to load bucket '{}'", name), e))?;

        let bucket = BucketItem {
            name: name.to_string(),
        };
        Ok(bucket)
    }

    pub async fn load_objects(&self, bucket: &str, prefix: &str) -> Result<Vec<ObjectItem>> {
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
        bucket: &str,
        key: &str,
        name: &str,
        size_byte: usize,
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
        let storage_class = output
            .storage_class()
            .map_or("", |s| s.as_str())
            .to_string();
        let key = key.to_owned();
        let s3_uri = self.build_s3_uri(bucket, &key);
        let arn = self.build_arn(bucket, &key);
        let object_url = self.build_object_url(bucket, &key);
        Ok(FileDetail {
            name,
            size_byte,
            last_modified,
            e_tag,
            content_type,
            storage_class,
            key,
            s3_uri,
            arn,
            object_url,
        })
    }

    fn build_s3_uri(&self, bucket: &str, key: &str) -> String {
        format!("s3://{}/{}", bucket, key)
    }

    fn build_arn(&self, bucket: &str, key: &str) -> String {
        format!("arn:aws:s3:::{}/{}", bucket, key)
    }

    fn build_object_url(&self, bucket: &str, key: &str) -> String {
        format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            bucket, self.region, key
        )
    }

    pub async fn load_object_versions(&self, bucket: &str, key: &str) -> Result<Vec<FileVersion>> {
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
                let size_byte = v.size().unwrap() as usize;
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

    pub async fn download_object<F>(
        &self,
        bucket: &str,
        key: &str,
        version_id: Option<String>,
        size_byte: usize,
        f: F,
    ) -> Result<RawObject>
    where
        F: Fn(usize),
    {
        let mut request = self.client.get_object().bucket(bucket).key(key);
        if let Some(version_id) = version_id {
            request = request.version_id(version_id);
        }

        let result = request.send().await;
        let output = result.map_err(|e| AppError::new("Failed to download object", e))?;

        let mut bytes: Vec<u8> = Vec::with_capacity(size_byte);
        let mut stream = output.body;
        let mut i = 0;
        while let Some(buf) = stream // buf: 32 KiB
            .try_next()
            .await
            .map_err(|e| AppError::new("Failed to collect body", e))?
        {
            bytes.extend(buf.to_vec());

            // suppress too many calls (32 KiB * 32 = 1 MiB)
            if i >= 32 {
                f(bytes.len());
                i = 0;
            }
            i += 1;
        }

        Ok(RawObject { bytes })
    }

    pub fn open_management_console_buckets(&self) -> Result<()> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/buckets?region={}",
            self.region
        );
        open::that(path).map_err(AppError::error)
    }

    pub fn open_management_console_list(&self, bucket: &str, prefix: &str) -> Result<()> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/buckets/{}?region={}&prefix={}",
            bucket, self.region, prefix
        );
        open::that(path).map_err(AppError::error)
    }

    pub fn open_management_console_object(
        &self,
        bucket: &str,
        prefix: &str,
        name: &str,
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
            ObjectItem::Dir { name }
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
            let size_byte = file.size().unwrap() as usize;
            let last_modified = convert_datetime(file.last_modified().unwrap());
            ObjectItem::File {
                name,
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
