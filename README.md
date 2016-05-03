# xi glium

**Xi glium** is a `glium` and `glium_text`-based interface on top of the
[**xi editor**](https://github.com/google/xi-editor) project back-end.

Screenshot:

![xi glium](/screenshot.png?raw=true)

## Features

* Write and backspace text
* Navigate using mouse, arrows, page-up and page-down
* Select text using keyboard
* cut, topy, paste, delete selection
* Load (`ctrl-o`), save (`ctrl-s`) and save-as (`ctrl-shift-s`) using GTK dialogs
* F1 to line-wrap

You must specify a path to the `xicore` executable (build by cargo inside
the `rust` subdirectory of xi-editor). Works with the xi-editor commit ` 7f7b885`,
but the HEAD is a good bet.

## Example usage

`xicore=../xi-editor/rust/target/debug/xicore cargo run README.md`
