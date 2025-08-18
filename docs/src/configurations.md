# Configurations

You can use `config.toml` to configure various aspects of your application.

Config is loaded from `$STU_ROOT_DIR/config.toml`.

- If `STU_ROOT_DIR` environment variable is not set, `~/.stu` is used by default.
  - If the `STU_ROOT_DIR` directory does not exist, it will be created automatically.
- If the config file does not exist, the default values will be used for all items.
- If the config file exists but some items are not set, the default values will be used for those unset items.

## STU_ROOT_DIR

`$STU_ROOT_DIR` will be structured as follows:

```
$STU_ROOT_DIR
│
├── config.toml
│
├── keybindings.toml
│
├── error.log
│
├── debug.log
│
├── preview_theme/
│   ├── material-theme-dark.tmTheme
│   └── ...
│
└── preview_syntax/
    ├── toml.sublime-syntax
    └── ...
```

----

- [Config File Format](./config-file-format.md)
- [Syntax Highlighting](./syntax-highlighting.md)
