# Configurations

You can use `config.toml` to configure various aspects of your application.

Config is loaded from `$STU_ROOT_DIR/config.toml`.

- If `STU_ROOT_DIR` environment variable is not set, `~/.stu` is used by default.
  - If the `STU_ROOT_DIR` directory does not exist, it will be created automatically.
- If the config file does not exist, the default values will be used for all items.
- If the config file exists but some items are not set, the default values will be used for those unset items.

