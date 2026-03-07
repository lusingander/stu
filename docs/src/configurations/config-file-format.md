# Config File Format

## Example

```toml
download_dir = "$STU_ROOT_DIR/download"

max_concurrent_requests = 5

default_region = "us-east-1"

[ui.bucket_list]
default_sort = "default"

[ui.object_list]
date_format = "%Y-%m-%d %H:%M:%S"
date_width = 19
default_sort = "default"

[ui.object_detail]
date_format = "%Y-%m-%d %H:%M:%S"

[ui.help]
max_help_width = 100

[ui.theme]
bg = "reset"
fg = "reset"
divider = "dark_gray"
link = "blue"
list_selected_bg = "#FFD166"
list_selected_fg = "black"
list_selected_inactive_bg = "dark_gray"
list_selected_inactive_fg = "black"
list_filter_match = "red"
detail_selected = "cyan"
dialog_selected = "cyan"
preview_line_number = "dark_gray"
help_key_fg = "yellow"
status_help = "dark_gray"
status_info = "blue"
status_success = "green"
status_warn = "yellow"
status_error = "red"
object_dir_bold = true

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

### `ui.bucket_list.default_sort`

The default sort order of the bucket list.

- type: `string`
- default: `default`
- possible values:
  - `default`
  - `name_asc`
  - `name_desc`

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

### `ui.object_list.default_sort`

The default sort order of the object list.

- type: `string`
- default: `default`
- possible values:
  - `default`
  - `name_asc`
  - `name_desc`
  - `date_asc`
  - `date_desc`
  - `size_asc`
  - `size_desc`

### `ui.object_detail.date_format`

The date format of a last modified in the object detail.
The format must be specified in [strftime format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html).

- type: `string`
- default: `%Y-%m-%d %H:%M:%S`

### `ui.help.max_help_width`

The maximum width of the keybindings display area in the help.

- type: `usize`
- default: `100`

### `ui.theme.bg`

The default background color used across the UI.

- type: `string`
- default: `reset`

### `ui.theme.fg`

The default foreground color used across the UI.

- type: `string`
- default: `reset`

### `ui.theme.divider`

The color of dividers and separators.

- type: `string`
- default: `dark_gray`

### `ui.theme.link`

The color used for links in the help view.

- type: `string`
- default: `blue`

### `ui.theme.list_selected_bg`

The background color of the selected row in bucket and object lists.

- type: `string`
- default: `cyan`

Theme colors are deserialized by Ratatui's `Color` serde support.
Supported examples include named colors such as `cyan`, `dark_gray`, `bright_white`,
hex colors such as `#FFD166`, and indexed colors such as `42`.

### `ui.theme.list_selected_fg`

The foreground color of the selected row in bucket and object lists.

- type: `string`
- default: `black`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.list_selected_inactive_bg`

The background color of the selected row when the list is inactive.

- type: `string`
- default: `dark_gray`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.list_selected_inactive_fg`

The foreground color of the selected row when the list is inactive.

- type: `string`
- default: `black`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.list_filter_match`

The color used to highlight matched filter text in lists.

- type: `string`
- default: `red`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.detail_selected`

The color used for the selected tab and selection accents in the object detail page.

- type: `string`
- default: `cyan`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.dialog_selected`

The color used to highlight the selected choice in dialogs.

- type: `string`
- default: `cyan`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.preview_line_number`

The color of line numbers in the text preview.

- type: `string`
- default: `dark_gray`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.help_key_fg`

The foreground color of key labels in the help view.

- type: `string`
- default: `yellow`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.status_help`

The color used for help/status hint messages.

- type: `string`
- default: `dark_gray`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.status_info`

The color used for informational status messages.

- type: `string`
- default: `blue`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.status_success`

The color used for success status messages.

- type: `string`
- default: `green`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.status_warn`

The color used for warning status messages.

- type: `string`
- default: `yellow`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.status_error`

The color used for error status messages.

- type: `string`
- default: `red`

Supports the same color formats as `ui.theme.list_selected_bg`.

### `ui.theme.object_dir_bold`

Whether directory names should be rendered in bold.

- type: `bool`
- default: `true`

### `preview.highlight`

Whether syntax highlighting is enabled in the object preview.

- type: `bool`
- default: `false`

See [Syntax Highlighting](./syntax-highlighting.md) for details on possible values.

### `preview.highlight_theme`

The name of the color theme to use for syntax highlighting in the object preview.

- type: `string`
- default: `base16-ocean.dark`

See [Color Themes](./syntax-highlighting.md#color-themes) for details on possible values.

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
