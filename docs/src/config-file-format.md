# Config File Format

## Example

```toml
download_dir = "$STU_ROOT_DIR/download"

max_concurrent_requests = 5

default_region = "us-east-1"

[ui.object_list]
date_format = "%Y-%m-%d %H:%M:%S"
date_width = 19

[ui.object_detail]
date_format = "%Y-%m-%d %H:%M:%S"

[ui.help]
max_help_width = 100

[preview]
highlight = false
highlight_theme = "base16-ocean.dark"

image = false

encodings = [
  "utf-8",
  "utf-16be",
  "utf-16le",
]
auto_detect_encoding = false
```

## Configuration Options

### `download_dir`

The directory to save the downloaded objects.

- type: `string`
- default: `$STU_ROOT_DIR/download`

`$STU_ROOT_DIR` will be the actual expanded value.

### `max_concurrent_requests`

The maximum number of concurrent requests when recursive downloading objects.

- type: `usize`
- default: `5`

### `default_region`

The default region to use if the region cannot be obtained from the command line options or AWS settings.

- type: `string`
- default: `us-east-1`

### `ui.object_list.date_format`

The date format of a last modified in the object list.
The format must be specified in [strftime format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

- type: `string`
- default: `%Y-%m-%d %H:%M:%S`

### `ui.object_list.date_width`

The width of a last modified in the object list.
It is recommended to set this when setting `date_format`.

- type: `u16`
- default: `19`

### `ui.object_detail.date_format`

The date format of a last modified in the object detail.
The format must be specified in [strftime format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

- type: `string`
- default: `%Y-%m-%d %H:%M:%S`

### `ui.help.max_help_width`

The maximum width of the keybindings display area in the help.

- type: `usize`
- default: `100`

### `preview.highlight`

Whether syntax highlighting is enabled in the object preview.

- type: `bool`
- default: `false`

### `preview.highlight_theme`

The name of the color theme to use for syntax highlighting in the object preview.

- type: `string`
- default: `base16-ocean.dark`

### `preview.image`

Whether image file preview is enabled in the object preview.

- type: `bool`
- default: `false`

### `preview.encodings`

Array of labels for the encoding want to use.
Label names should be specified from [https://encoding.spec.whatwg.org/#names-and-labels](https://encoding.spec.whatwg.org/#names-and-labels).

- type: `array of strings`
- default: `["utf-8", "utf-16be", "utf-16le"]`

### `preview.auto_detect_encoding`

Whether to enable encoding auto detection.

- type: `bool`
- default: `false`
