Set Shell zsh

Output ${OUTPUT_DIR}/screenshot.gif

Set FontSize 15
Set Width 1000
Set Height 600
Set Padding 15
Set Theme "nord"
Set TypingSpeed 20ms

Type@10ms "export AWS_ACCESS_KEY_ID=dummy"
Enter
Type@10ms "export AWS_SECRET_ACCESS_KEY=dummy"
Enter
Type@10ms "export STU_ROOT_DIR=./tool/imggen/test_root_dir"
Enter
Type@10ms "./target/debug/stu --endpoint-url ${ENDPOINT_URL}"
Enter
Sleep 3s

# bucket list
Type "j"
Sleep 100ms
Type "j"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list.png
Type "k"
Sleep 100ms
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list-help.png
Type "?"
Sleep 100ms
Type "o"
Sleep 100ms
Type "j"
Sleep 100ms
Type "j"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list-sort.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list-sort-help.png
Type "?"
Sleep 100ms
Escape
Sleep 100ms
Type "/"
Sleep 100ms
Type@10ms "buck"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list-filter.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list-filter-help.png
Type "?"
Sleep 100ms
Escape
Sleep 100ms
Type "j"
Sleep 100ms
Type "r"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list-copy.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/bucket-list-copy-help.png
Type "?"
Sleep 100ms
Escape
Sleep 100ms
Type "s"
Sleep 500ms
Screenshot ${OUTPUT_DIR}/bucket-list-download-confirm.png
Sleep 100ms
Escape

# open test-bucket-2
Enter
Sleep 500ms
Screenshot ${OUTPUT_DIR}/object-list-simple.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-help.png
Type "?"
Sleep 100ms
Type "o"
Sleep 100ms
Type "k"
Sleep 100ms
Type "k"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-sort.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-sort-help.png
Type "?"
Sleep 100ms
Enter
Sleep 100ms
Type "/"
Sleep 100ms
Type@10ms "image"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-filter.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-filter-help.png
Type "?"
Sleep 100ms
Enter

# open image.jpeg
Type "j"
Sleep 100ms
Enter
Sleep 500ms
Screenshot ${OUTPUT_DIR}/object-detail.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-detail-help.png
Type "?"
Sleep 100ms
Type "l"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-version.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-version-help.png
Type "?"
Sleep 100ms
Type "p"
Sleep 1s
Screenshot ${OUTPUT_DIR}/object-preview-image.png
Sleep 300ms
Backspace
Sleep 100ms
Backspace
Sleep 100ms
Escape
Sleep 100ms
Type "j"
Sleep 100ms
Type "j"
Sleep 100ms
Type "j"
Sleep 100ms

# open medium.txt
Enter
Sleep 100ms
Type "r"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-details-copy.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-details-copy-help.png
Type "?"
Sleep 100ms
Escape
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
Sleep 100ms
Enter
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-download-success.png
Enter
Sleep 100ms

Backspace
Sleep 100ms
Backspace
Sleep 100ms
Backspace
Sleep 100ms

Type "~"
Sleep 100ms
Type "g"
Sleep 100ms
Enter

# open test-bucket-1
Sleep 500ms
Type "S"
Sleep 1s
Screenshot ${OUTPUT_DIR}/object-list-download-confirm.png
Sleep 500ms
Enter
Sleep 100ms
Type "baaar"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-download.png
Escape
Sleep 500ms
Type "j"
Sleep 100ms
Type "j"
Sleep 100ms
Enter
Sleep 500ms
Type "j"
Sleep 100ms
Type "r"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-dir-copy.png
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-copy-help.png
Type "?"
Sleep 100ms
Escape
Sleep 100ms
Type "j"
Sleep 100ms
Enter
Sleep 500ms
Enter
Sleep 500ms
Enter
Sleep 500ms
Screenshot ${OUTPUT_DIR}/object-list-hierarchy.png

Backspace
Type "j"
Sleep 100ms
Type "j"
Sleep 100ms
Type "j"
Sleep 100ms
Type "r"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-list-file-copy.png
Sleep 100ms
Escape
Sleep 500ms
Enter
Sleep 500ms
Type "p"
Sleep 1s
Type@10ms "jjjjj"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-preview.png
Sleep 100ms
Type "e"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-preview-encoding.png
Escape
Sleep 100ms
Type "?"
Sleep 100ms
Screenshot ${OUTPUT_DIR}/object-preview-help.png
Type "?"
Sleep 100ms
Backspace
Sleep 100ms
Backspace
Sleep 100ms

Type "~"
Sleep 100ms
Type "g"
Sleep 100ms
Type "j"
Sleep 100ms
Type "j"
Sleep 100ms
Enter
Sleep 100ms
Screenshot ${OUTPUT_DIR}/loading-object-list.png
Sleep 5s
Screenshot ${OUTPUT_DIR}/object-list-many.png