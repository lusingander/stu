use std::fs;

use anyhow::{ensure, Context, Result};
use aws_sdk_s3::config::{Credentials, Region};
use tempfile::{tempdir, TempDir};
use termlens::{Key, Screen, Terminal};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    ContainerAsync, GenericImage, ImageExt,
};

pub struct Storage {
    pub endpoint: String,
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

        for bucket in buckets {
            client
                .create_bucket()
                .bucket(*bucket)
                .send()
                .await
                .with_context(|| format!("Failed to create fixture bucket '{bucket}'"))?;
        }

        Ok(Self {
            endpoint,
            _container: container,
        })
    }
}

pub struct TestApp {
    pub terminal: Terminal,
    directory: TempDir,
    exited: bool,
}

impl TestApp {
    pub fn spawn(endpoint: &str, args: &[&str]) -> Result<Self> {
        let directory = tempdir()?;
        fs::write(
            directory.path().join("config.toml"),
            r#"
[ui.bucket_list]
default_sort = "name_asc"

[ui.object_list]
default_sort = "name_asc"

[preview]
image = false
"#,
        )?;
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
