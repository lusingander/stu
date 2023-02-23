use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::Region;

use crate::app::Item;

const DELIMITER: &str = "/";
const DEFAULT_REGION: &str = "ap-northeast-1";

pub struct Client {
    pub client: aws_sdk_s3::Client,
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
        Client { client }
    }

    pub async fn load_all_buckets(&self) -> Vec<Item> {
        let result = self.client.list_buckets().send().await;
        let output = result.unwrap();

        let buckets = output.buckets().unwrap_or_default();
        buckets
            .iter()
            .map(|bucket| {
                let name = bucket.name().unwrap().to_string();
                Item::Bucket { name }
            })
            .collect()
    }

    pub async fn load_objects(&self, bucket: &String, prefix: &String) -> Vec<Item> {
        let result = self
            .client
            .list_objects_v2()
            .bucket(bucket)
            .prefix(prefix)
            .delimiter(DELIMITER)
            .send()
            .await;
        let output = result.unwrap();

        let objects = output.common_prefixes().unwrap_or_default();
        let dirs = objects.iter().map(|dir| {
            let path = dir.prefix().unwrap().to_string();
            let paths = parse_path(&path, true);
            Item::Dir {
                name: paths.last().unwrap().to_owned(),
                paths,
            }
        });

        let objects = output.contents().unwrap_or_default();
        let files = objects.iter().map(|file| {
            let path = file.key().unwrap().to_string();
            let paths = parse_path(&path, false);
            Item::File {
                name: paths.last().unwrap().to_owned(),
                paths,
            }
        });

        dirs.chain(files).collect()
    }
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
