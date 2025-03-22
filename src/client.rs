use std::{
    fmt::Debug,
    future::Future,
    io::{BufWriter, Write},
};

use aws_config::{default_provider::region, meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_s3::{
    config::Region,
    error::SdkError,
    operation::list_objects_v2::{ListObjectsV2Error, ListObjectsV2Output},
};
use chrono::TimeZone;

use crate::{
    error::{AppError, Result},
    object::{BucketItem, DownloadObjectInfo, FileDetail, FileVersion, ObjectItem},
};

const DELIMITER: &str = "/";

pub enum AddressingStyle {
    Auto,
    Path,          // https://s3.region.amazonaws.com/bucket/key
    VirtualHosted, // https://bucket.s3.region.amazonaws.com/key
}

impl AddressingStyle {
    fn to_force_path_style(&self, endpoint_url: Option<&str>) -> bool {
        match self {
            AddressingStyle::Auto => endpoint_url.is_some(),
            AddressingStyle::Path => true,
            AddressingStyle::VirtualHosted => false,
        }
    }
}

#[rustfmt::skip]
pub trait Client: Send + Sync + 'static + Debug {
    fn region(&self) -> &str;
    fn load_all_buckets(&self) -> impl Future<Output = Result<Vec<BucketItem>>> + Send;
    fn load_bucket(&self, name: &str) -> impl Future<Output = Result<Vec<BucketItem>>> + Send;
    fn load_objects(&self, bucket: &str, prefix: &str) -> impl Future<Output = Result<Vec<ObjectItem>>> + Send;
    fn load_object_detail(&self, bucket: &str, key: &str, name: &str) -> impl Future<Output = Result<FileDetail>> + Send;
    fn load_object_versions(&self, bucket: &str, key: &str) -> impl Future<Output = Result<Vec<FileVersion>>> + Send;
    fn download_object<W: std::io::Write + Send, F: Fn(usize) + Send>(&self, bucket: &str, key: &str, version_id: Option<String>, writer: &mut BufWriter<W>, f: F) -> impl Future<Output = Result<()>> + Send;
    fn list_all_download_objects(&self, bucket: &str, prefix: &str) -> impl Future<Output = Result<Vec<DownloadObjectInfo>>> + Send;
    fn open_management_console_buckets(&self) -> Result<()>;
    fn open_management_console_list(&self, bucket: &str, prefix: &str) -> Result<()>;
    fn open_management_console_object(&self, bucket: &str, prefix: &str) -> Result<()>;
}

pub async fn new(
    region: Option<String>,
    endpoint_url: Option<String>,
    profile: Option<String>,
    default_region_fallback: String,
    addressing_style: AddressingStyle,
) -> impl Client {
    AwsSdkClient::new(
        region,
        endpoint_url,
        profile,
        default_region_fallback,
        addressing_style,
    )
    .await
}

struct AwsSdkClient {
    client: aws_sdk_s3::Client,
    region: String,
}

impl Debug for AwsSdkClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AwsSdkClient {{ region: {} }}", self.region)
    }
}

impl AwsSdkClient {
    async fn new(
        region: Option<String>,
        endpoint_url: Option<String>,
        profile: Option<String>,
        default_region_fallback: String,
        addressing_style: AddressingStyle,
    ) -> AwsSdkClient {
        let mut region_builder = region::Builder::default();
        if let Some(profile) = &profile {
            region_builder = region_builder.profile_name(profile);
        }
        let region_provider = RegionProviderChain::first_try(region.map(Region::new))
            .or_else(region_builder.build())
            .or_else(Region::new(default_region_fallback));

        let mut config_loader =
            aws_config::defaults(BehaviorVersion::latest()).region(region_provider);
        if let Some(url) = &endpoint_url {
            config_loader = config_loader.endpoint_url(url);
        }
        if let Some(profile) = &profile {
            config_loader = config_loader.profile_name(profile);
        }
        let sdk_config = config_loader.load().await;

        let config_builder = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(addressing_style.to_force_path_style(sdk_config.endpoint_url()));
        let config = config_builder.build();

        let client = aws_sdk_s3::Client::from_conf(config);
        let region = sdk_config.region().unwrap().to_string();

        AwsSdkClient { client, region }
    }
}

impl Client for AwsSdkClient {
    fn region(&self) -> &str {
        &self.region
    }

    async fn load_all_buckets(&self) -> Result<Vec<BucketItem>> {
        let list_buckets_result = self
            .client
            .list_buckets()
            .bucket_region(&self.region)
            .send()
            .await;
        let list_buckets_output =
            list_buckets_result.map_err(|e| AppError::new("Failed to load buckets", e))?;

        let buckets: Vec<BucketItem> = list_buckets_output
            .buckets()
            .iter()
            .map(|bucket| {
                let bucket_name = bucket.name().unwrap().to_string();
                let s3_uri = build_bucket_s3_uri(&bucket_name);
                let arn = build_bucket_arn(&bucket_name);
                let object_url = build_bucket_url(&self.region, &bucket_name);
                BucketItem {
                    name: bucket_name,
                    s3_uri,
                    arn,
                    object_url,
                }
            })
            .collect();

        if buckets.is_empty() {
            Err(AppError::msg("No buckets found"))
        } else {
            Ok(buckets)
        }
    }

    async fn load_bucket(&self, name: &str) -> Result<Vec<BucketItem>> {
        let s3_uri = build_bucket_s3_uri(name);
        let arn = build_bucket_arn(name);
        let object_url = build_bucket_url(&self.region, name);

        let bucket = BucketItem {
            name: name.to_string(),
            s3_uri,
            arn,
            object_url,
        };
        Ok(vec![bucket])
    }

    async fn load_objects(&self, bucket: &str, prefix: &str) -> Result<Vec<ObjectItem>> {
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

            if let Err(SdkError::ServiceError(ref e)) = result {
                if let ListObjectsV2Error::NoSuchBucket(_) = e.err() {
                    return Err(AppError::msg(format!("Bucket '{}' not found", bucket)));
                }
            }

            let output = result.map_err(|e| AppError::new("Failed to load objects", e))?;

            let dirs = objects_output_to_dirs(&self.region, bucket, &output);
            dirs_vec.push(dirs);

            let files = objects_output_to_files(&self.region, bucket, &output);
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

    async fn load_object_detail(&self, bucket: &str, key: &str, name: &str) -> Result<FileDetail> {
        let result = self
            .client
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await;
        let output = result.map_err(|e| AppError::new("Failed to load object detail", e))?;

        let name = name.to_owned();
        let size_byte = output.content_length().unwrap() as usize;
        let last_modified = convert_datetime(output.last_modified().unwrap());
        let e_tag = output.e_tag().unwrap().trim_matches('"').to_string();
        let content_type = output.content_type().unwrap().to_string();
        let storage_class = output
            .storage_class()
            .map_or("", |s| s.as_str())
            .to_string();
        let key = key.to_owned();
        let s3_uri = build_object_s3_uri(bucket, &key);
        let arn = build_object_arn(bucket, &key);
        let object_url = build_object_url(&self.region, bucket, &key);
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

    async fn load_object_versions(&self, bucket: &str, key: &str) -> Result<Vec<FileVersion>> {
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
                let e_tag = v.e_tag().unwrap().trim_matches('"').to_string();
                let is_latest = v.is_latest().unwrap();
                FileVersion {
                    version_id,
                    size_byte,
                    last_modified,
                    e_tag,
                    is_latest,
                }
            })
            .collect();
        Ok(versions)
    }

    async fn download_object<W: std::io::Write + Send, F: Fn(usize) + Send>(
        &self,
        bucket: &str,
        key: &str,
        version_id: Option<String>,
        writer: &mut BufWriter<W>,
        f: F,
    ) -> Result<()> {
        let mut request = self.client.get_object().bucket(bucket).key(key);
        if let Some(version_id) = version_id {
            request = request.version_id(version_id);
        }

        let result = request.send().await;
        let output = result.map_err(|e| AppError::new("Failed to download object", e))?;

        let mut stream = output.body;
        let mut i = 0;
        let mut total_bytes = 0;
        while let Some(buf) = stream // buf: 32 KiB
            .try_next()
            .await
            .map_err(|e| AppError::new("Failed to collect body", e))?
        {
            writer.write_all(&buf).map_err(AppError::error)?;
            total_bytes += buf.len();

            // suppress too many calls (32 KiB * 32 = 1 MiB)
            if i >= 32 {
                f(total_bytes);
                i = 0;
            }
            i += 1;
        }
        f(total_bytes); // last call

        writer.flush().map_err(AppError::error)?;
        Ok(())
    }

    async fn list_all_download_objects(
        &self,
        bucket: &str,
        prefix: &str,
    ) -> Result<Vec<DownloadObjectInfo>> {
        let mut objs: Vec<DownloadObjectInfo> = Vec::new();

        let mut token: Option<String> = None;
        loop {
            let result = self
                .client
                .list_objects_v2()
                .bucket(bucket)
                .prefix(prefix)
                .set_continuation_token(token)
                .send()
                .await;
            let output = result.map_err(|e| AppError::new("Failed to list download objects", e))?;

            let os = output
                .contents()
                .iter()
                .map(|file| {
                    let key = file.key().unwrap().to_owned();
                    let size_byte = file.size().unwrap() as usize;
                    DownloadObjectInfo { key, size_byte }
                })
                .filter(|f| !f.key.ends_with('/')); // skip dummy empty object
            objs.extend(os);

            token = output.next_continuation_token().map(String::from);
            if token.is_none() {
                break;
            }
        }

        Ok(objs)
    }

    fn open_management_console_buckets(&self) -> Result<()> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/buckets?region={}",
            self.region
        );
        open::that(path).map_err(AppError::error)
    }

    fn open_management_console_list(&self, bucket: &str, prefix: &str) -> Result<()> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/buckets/{}?region={}&prefix={}",
            bucket, self.region, prefix
        );
        open::that(path).map_err(AppError::error)
    }

    fn open_management_console_object(&self, bucket: &str, prefix: &str) -> Result<()> {
        let path = format!(
            "https://s3.console.aws.amazon.com/s3/object/{}?region={}&prefix={}",
            bucket, self.region, prefix
        );
        open::that(path).map_err(AppError::error)
    }
}

fn objects_output_to_dirs(
    region: &str,
    bucket: &str,
    output: &ListObjectsV2Output,
) -> Vec<ObjectItem> {
    let objects = output.common_prefixes();
    objects
        .iter()
        .map(|dir| {
            let path = dir.prefix().unwrap();
            let paths = parse_path(path, true);
            let name = paths.last().unwrap().to_owned();

            let key = path.to_owned();
            let s3_uri = build_object_s3_uri(bucket, &key);
            let object_url = build_object_url(region, bucket, &key);

            ObjectItem::Dir {
                name,
                key,
                s3_uri,
                object_url,
            }
        })
        .filter(|f| !f.name().is_empty()) // skip dummy empty object
        .collect()
}

fn objects_output_to_files(
    region: &str,
    bucket: &str,
    output: &ListObjectsV2Output,
) -> Vec<ObjectItem> {
    let objects = output.contents();
    objects
        .iter()
        .map(|file| {
            let path = file.key().unwrap();
            let paths = parse_path(path, false);
            let name = paths.last().unwrap().to_owned();
            let size_byte = file.size().unwrap() as usize;
            let last_modified = convert_datetime(file.last_modified().unwrap());

            let key = file.key().unwrap().to_owned();
            let s3_uri = build_object_s3_uri(bucket, &key);
            let arn = build_object_arn(bucket, &key);
            let object_url = build_object_url(region, bucket, &key);
            let e_tag = file.e_tag().unwrap().trim_matches('"').to_string();

            ObjectItem::File {
                name,
                size_byte,
                last_modified,
                key,
                s3_uri,
                arn,
                object_url,
                e_tag,
            }
        })
        .filter(|f| !f.name().is_empty()) // skip dummy empty object
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

fn build_bucket_s3_uri(bucket: &str) -> String {
    format!("s3://{}/", bucket)
}

fn build_bucket_arn(bucket: &str) -> String {
    format!("arn:aws:s3:::{}", bucket)
}

fn build_bucket_url(region: &str, bucket: &str) -> String {
    format!("https://{}.s3.{}.amazonaws.com/", bucket, region)
}

fn build_object_s3_uri(bucket: &str, key: &str) -> String {
    format!("s3://{}/{}", bucket, key)
}

fn build_object_arn(bucket: &str, key: &str) -> String {
    format!("arn:aws:s3:::{}/{}", bucket, key)
}

fn build_object_url(region: &str, bucket: &str, key: &str) -> String {
    format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, key)
}
