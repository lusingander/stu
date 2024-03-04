CMD_DIR=./tool/imggen
OUTPUT_DIR=./dist

.PHONY: screenshot
screenshot:
	go run $(CMD_DIR)/*.go generate -tape $(CMD_DIR)/tape/screenshot.tape -out $(OUTPUT_DIR)/screenshot

.PHONY: clean
clean:
	rm -rf $(OUTPUT_DIR)
	