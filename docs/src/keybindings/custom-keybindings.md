# Custom Keybindings

You can set your own custom key bindings.

Custom bindings are loaded from `$STU_ROOT_DIR/keybindings.toml`.

The default key bindings are defined in [./assets/keybindings.toml](https://github.com/lusingander/stu/blob/master/assets/keybindings.toml). You can set key bindings for each screen action in the same format.

- It is possible to set multiple key bindings for one action.
- If you do not set key bindings for an action, the default key bindings will be assigned.
- You can disable an action by setting `[]` as the key bindings.

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
