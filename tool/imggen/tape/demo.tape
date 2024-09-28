Set Shell zsh

Output ${OUTPUT_DIR}/demo.gif

Set FontSize 15
Set Width 1000
Set Height 600
Set Padding 15
Set WindowBar Colorful
Set Theme "nord"

Hide

Type@10ms "export AWS_ACCESS_KEY_ID=dummy"
Enter
Type@10ms "export AWS_SECRET_ACCESS_KEY=dummy"
Enter
Type@10ms "export STU_ROOT_DIR=./tool/imggen/test_root_dir"
Enter
Type@10ms "./target/debug/stu --endpoint-url ${ENDPOINT_URL}"
Enter
Sleep 3s

Show

Sleep 1s
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "k"
Sleep 500ms

# open test-bucket-2
Enter
Sleep 1s
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 500ms
# sort
Type "o"
Sleep 500ms
Type "k"
Sleep 500ms
Type "k"
Sleep 500ms
Enter
Sleep 500ms
# filter
Type "/"
Sleep 500ms
Type@100ms "image"
Enter
Sleep 500ms
Type "j"
Sleep 300ms

# open image.jpeg
Enter
Sleep 1.5s
Type "l"
Sleep 1s
Type "j"
Sleep 500ms
Type "j"
Sleep 500ms
Type "p"
Sleep 2s
Type "s"
Sleep 1s
Backspace
Type "h"
Sleep 1s
Backspace
Sleep 700ms
Backspace
Sleep 500ms
Type "g"
Sleep 500ms

# open test-bucket-1
Enter
Sleep 1s
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Enter
Sleep 1s
Type "j"
Sleep 500ms
Enter
Sleep 1s
Backspace
Sleep 500ms
Type "j"
Sleep 500ms
Enter
Sleep 1s
Enter
Sleep 1s
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# open foo/quux/fred/file_1.rs
Enter
Sleep 1s
Type "p"
Sleep 2s
Type@200ms "jjjjj"
Sleep 500ms
Type "f"
Sleep 500ms
Type "S"
Sleep 500ms
Type@100ms "object.rs"
Sleep 500ms
Enter
Sleep 3s
