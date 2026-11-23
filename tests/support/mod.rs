use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{ensure, Context, Result};
use aws_sdk_s3::config::{Credentials, Region};
use tempfile::{tempdir, TempDir};
use termlens::{Color, Key, Screen, Terminal};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};
use tokio::time::{sleep, timeout};

pub struct Storage {
    pub endpoint: String,
    client: aws_sdk_s3::Client,
    _container: ContainerAsync<GenericImage>,
}

impl Storage {
    pub async fn new(buckets: &[&str]) -> Result<Self> {
        // Use the same pinned image as the VHS fixtures, without an auth token.
        let container = GenericImage::new("localstack/localstack", "3.2.0")
            .with_exposed_port(4566.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Ready."))
            .with_env_var("SERVICES", "s3")
            .start()
            .await
            .context(
                "Failed to start LocalStack; terminal E2E tests require a running Docker daemon",
            )?;
        let host = container.get_host().await?;
        let port = container.get_host_port_ipv4(4566).await?;
        let endpoint = format!("http://{host}:{port}");
        let config = aws_sdk_s3::Config::builder()
            .behavior_version_latest()
            .region(Region::new("us-east-1"))
            .credentials_provider(Credentials::new("test", "test", None, None, "e2e"))
            .endpoint_url(&endpoint)
            .force_path_style(true)
            .build();
        let client = aws_sdk_s3::Client::from_conf(config);

        // The ready log can precede host port availability. Check S3 itself
        // before creating fixtures, using the SDK already needed to seed them.
        timeout(Duration::from_secs(30), async {
            while client.list_buckets().send().await.is_err() {
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .with_context(|| format!("LocalStack S3 did not become ready at {endpoint}"))?;

        let storage = Self {
            endpoint,
            client,
            _container: container,
        };
        for bucket in buckets {
            storage.create_bucket(bucket).await?;
        }
        Ok(storage)
    }

    pub async fn create_bucket(&self, bucket: &str) -> Result<()> {
        self.client
            .create_bucket()
            .bucket(bucket)
            .send()
            .await
            .with_context(|| {
                format!(
                    "Failed to create fixture bucket '{bucket}' at {}",
                    self.endpoint
                )
            })?;
        Ok(())
    }

    pub async fn put_object(&self, bucket: &str, key: &str, body: &[u8]) -> Result<()> {
        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .content_type("text/plain")
            .body(body.to_vec().into())
            .send()
            .await
            .with_context(|| format!("Failed to create fixture object 's3://{bucket}/{key}'"))?;
        Ok(())
    }
}

pub struct TestApp {
    pub terminal: Terminal,
    directory: TempDir,
    exited: bool,
}

impl TestApp {
    pub fn spawn(endpoint: &str, args: &[&str]) -> Result<Self> {
        Self::spawn_with_config(
            endpoint,
            args,
            r#"
[ui.bucket_list]
default_sort = "name_asc"

[ui.object_list]
default_sort = "name_asc"

[preview]
image = false
"#,
        )
    }

    pub fn spawn_with_config(endpoint: &str, args: &[&str], config: &str) -> Result<Self> {
        let directory = tempdir()?;
        fs::write(directory.path().join("config.toml"), config)?;
        let terminal = termlens::bin!(
            "stu",
            size(100, 30),
            current_dir(directory.path()),
            env("STU_ROOT_DIR", directory.path()),
            env("AWS_ACCESS_KEY_ID", "test"),
            env("AWS_SECRET_ACCESS_KEY", "test"),
            env("AWS_EC2_METADATA_DISABLED", "true"),
            env("AWS_CONFIG_FILE", directory.path().join("aws-config")),
            env(
                "AWS_SHARED_CREDENTIALS_FILE",
                directory.path().join("aws-credentials")
            ),
            env("TZ", "UTC"),
            args([
                "--endpoint-url",
                endpoint,
                "--region",
                "us-east-1",
                "--debug"
            ]),
            args(args)
        )?;

        Ok(Self {
            terminal,
            directory,
            exited: false,
        })
    }

    pub fn quit(mut self) -> Result<()> {
        self.terminal.send(Key::Ctrl('c'))?;
        let status = self.terminal.wait_exit()?;
        ensure!(status.success(), "stu exit status: {status}");
        let screen = self.terminal.screen();
        ensure!(
            !screen.alternate_screen(),
            "Alternate screen was not restored"
        );
        ensure!(screen.cursor_visible(), "Cursor was not restored");
        self.exited = true;
        Ok(())
    }

    pub fn downloaded_files(&self) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
        self.files_in("download")
    }

    pub fn files_in(&self, directory: &str) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
        fn collect_files(
            root: &Path,
            directory: &Path,
            files: &mut BTreeMap<PathBuf, Vec<u8>>,
        ) -> Result<()> {
            for entry in fs::read_dir(directory)? {
                let entry = entry?;
                let path = entry.path();
                if entry.file_type()?.is_dir() {
                    collect_files(root, &path, files)?;
                } else {
                    files.insert(path.strip_prefix(root)?.to_path_buf(), fs::read(&path)?);
                }
            }
            Ok(())
        }

        let directory = self.directory.path().join(directory);
        let mut files = BTreeMap::new();
        // An unused destination may not exist.
        if directory.try_exists()? {
            collect_files(&directory, &directory, &mut files)?;
        }
        Ok(files)
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.exited {
            for name in ["error.log", "debug.log"] {
                if let Ok(log) = fs::read_to_string(self.directory.path().join(name)) {
                    let mut lines: Vec<_> = log.lines().rev().take(40).collect();
                    lines.reverse();
                    eprintln!("stu {name} (last 40 lines):\n{}", lines.join("\n"));
                }
            }
        }
    }
}

pub fn at_path(screen: &Screen, path: &str) -> bool {
    screen.row_text(1).trim_matches(['│', ' ']) == path && !screen.contains("Loading...")
}

pub fn footer_contains(screen: &Screen, text: &str) -> bool {
    (screen.rows().saturating_sub(2)..screen.rows()).any(|row| screen.row_text(row).contains(text))
}

pub fn list_rows(screen: &Screen) -> Vec<String> {
    // For list pages without a dialog: exclude the three-row header,
    // the list's top and bottom borders, and the two-row footer.
    (4..screen.rows().saturating_sub(3))
        .map(|row| screen.row_text(row).trim_matches(['│', ' ']).to_string())
        .filter(|row| !row.is_empty())
        .collect()
}

pub fn selected_item(screen: &Screen, name: &str) -> bool {
    (0..screen.rows()).any(|row| {
        screen
            .row_text(row)
            .trim_matches(['│', ' '])
            .starts_with(name)
            && screen
                .cell(row, 3)
                .is_some_and(|cell| cell.style().bg == Color::Indexed(6))
    })
}
