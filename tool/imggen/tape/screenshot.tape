Set Shell zsh

Output ${OUTPUT_DIR}/screenshot.gif

Set FontSize 15
Set Width 1000
Set Height 600
Set Padding 15
Set BorderRadius 20

Type@10ms "export AWS_ACCESS_KEY_ID=dummy"
Enter
Type@10ms "export AWS_SECRET_ACCESS_KEY=dummy"
Enter
Type@10ms "export STU_ROOT_DIR=./tool/imggen"
Enter
Type@10ms "./target/debug/stu --endpoint-url ${ENDPOINT_URL}"
Enter
Sleep 500ms
Screenshot ${OUTPUT_DIR}/initialize.png
Sleep 2s

Type "j"
Type "j"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list.png
Type "k"
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list-help.png
Type "?"

Enter
Sleep 500ms
Screenshot ${OUTPUT_DIR}/object-list-simple.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-help.png
Type "?"

Type "j"
Type "j"
Enter
Sleep 500ms
Screenshot ${OUTPUT_DIR}/object-detail.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-detail-help.png
Type "?"
Type "l"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-version.png
Backspace
Type "j"
Type "j"
Type "j"
Enter
Type "r"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-details-copy.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-details-copy-help.png
Type "?"
Backspace
Sleep 500ms

Type "S"
Sleep 100ms
Type@10ms "download_file.txt"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-download.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-download-help.png
Type "?"
Enter
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-download-success.png
Enter
Sleep 100ms

Type "p"
Sleep 500ms
Type@10ms "jjjjjjj"
Screenshot ${OUTPUT_DIR}/object-preview.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-preview-help.png
Type "?"
Type "s"
Sleep 500ms
Backspace
Backspace
Backspace

Type "~"
Type "g"
Enter
Sleep 500ms
Type "j"
Type "j"
Enter
Sleep 500ms
Type "j"
Type "j"
Enter
Sleep 500ms
Enter
Sleep 500ms
Enter
Sleep 500ms
Screenshot ${OUTPUT_DIR}/object-list-hierarchy.png

Type "~"
Type "g"
Type "j"
Type "j"
Enter
Sleep 100ms
Screenshot ${OUTPUT_DIR}/loading-object-list.png
Sleep 5s
Screenshot ${OUTPUT_DIR}/object-list-many.png