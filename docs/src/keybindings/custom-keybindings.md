# Custom Keybindings

You can set your own custom key bindings.

Custom bindings are loaded from `$STU_ROOT_DIR/keybindings.toml`.

The default key bindings are defined in [./assets/keybindings.toml](https://github.com/lusingander/stu/blob/master/assets/keybindings.toml). You can set key bindings for each screen action in the same format.

- It is possible to set multiple key bindings for one action.
- If you do not set key bindings for an action, the default key bindings will be assigned.
- You can disable an action by setting `[]` as the key bindings.

## Key Formats

You can use the following formats to define key bindings.

### Modifier Keys

- `ctrl-`
- `alt-`
- `shift-`

Modifiers can be combined, for example: `ctrl-shift-a`.

### Special Keys

| Key | Description |
| --- | --- |
| `esc` | Escape |
| `enter` | Enter |
| `left` | Left arrow |
| `right` | Right arrow |
| `up` | Up arrow |
| `down` | Down arrow |
| `home` | Home |
| `end` | End |
| `pageup` | Page Up |
| `pagedown` | Page Down |
| `backtab` | Back Tab (Shift + Tab) |
| `backspace` | Backspace |
| `delete` | Delete |
| `insert` | Insert |
| `f1` - `f12` | Function keys |
| `space` | Space |
| `hyphen`, `minus` | Hyphen (-) |
| `tab` | Tab |

### Character Keys

Any single character not listed above (e.g., `a`, `b`, `1`, `!`) can be used as a key.

## Example

If you want to change the key bindings for moving up and down from `k`/`j` to `up`/`down`, you can set the following in `$STU_ROOT_DIR/keybindings.toml`.

```toml
[bucket_list]
down = ["down"]
up = ["up"]

[object_list]
down = ["down"]
up = ["up"]

[object_detail]
down = ["down"]
up = ["up"]

[object_preview]
down = ["down"]
up = ["up"]

[select_dialog]
down = ["down"]
up = ["up"]
```
