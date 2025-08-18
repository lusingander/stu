# Syntax Highlighting

In the object preview, Syntax highlighting using [syntect](https://github.com/trishume/syntect) is available.

To enable this, set [`preview.highlight = true`](./config-file-format.md#previewhighlight) in the config file.

## Color Themes

You can change the color theme by specifying the theme name in [`preview.highlight_theme`](./config-file-format.md#previewhighlight_theme).

By default the following themes are available:

- `base16-ocean.dark`
- `base16-eighties.dark`
- `base16-mocha.dark`
- `base16-ocean.light`
  - [https://github.com/SublimeText/Spacegray](https://github.com/SublimeText/Spacegray)
- `InspiredGitHub`
  - [https://github.com/sethlopez/InspiredGitHub.tmtheme](https://github.com/sethlopez/InspiredGitHub.tmtheme)
- `Solarized (dark)`
- `Solarized (light)`
  - [https://github.com/altercation/solarized](https://github.com/altercation/solarized)

Also, by creating `xxx.tmTheme` in `$STU_ROOT_DIR/preview_theme/`, you can use `xxx` and load it.

## Syntax definitions

You can add syntax definitions for file types that are not supported by default.

You can use it by creating a `.sublime-syntax` file in `$STU_ROOT_DIR/preview_syntax/`.

[https://www.sublimetext.com/docs/syntax.html](https://www.sublimetext.com/docs/syntax.html)
