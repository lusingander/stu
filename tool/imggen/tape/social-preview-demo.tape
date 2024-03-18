# For GitHub social media preview
Set Shell zsh

Output ${OUTPUT_DIR}/social-preview-demo.gif

Set FontSize 15
Set Width 1280
Set Height 640
Set Padding 10

Hide

Type@10ms "export AWS_ACCESS_KEY_ID=dummy"
Enter
Type@10ms "export AWS_SECRET_ACCESS_KEY=dummy"
Enter
Type@10ms "export STU_ROOT_DIR=./tool/imggen"
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

Enter
Sleep 1s
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Enter
Sleep 1.5s
Type "l"
Sleep 1s
Type "h"
Sleep 1s
Backspace
Sleep 700ms
Backspace
Sleep 500ms
Type "g"
Sleep 500ms

Enter
Sleep 1s
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Enter
Sleep 500ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Enter
Sleep 500ms
Enter
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Enter
Sleep 1s
Type "p"
Sleep 1s
Type@200ms "jjjjjjj"
Sleep 500ms
Type "S"
Sleep 500ms
Type@100ms "lamento.txt"
Sleep 500ms
Enter
Sleep 2s
