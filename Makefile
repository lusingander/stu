CMD_DIR=./tool
IMGGEN_DIR=$(CMD_DIR)/imggen
IMGDIFF_DIR=$(CMD_DIR)/imgdiff
OUTPUT_DIR=./dist

.PHONY: demo
demo:
	go run $(IMGGEN_DIR)/*.go generate -tape $(IMGGEN_DIR)/tape/demo.tape -out $(OUTPUT_DIR)/demo

.PHONY: screenshot
screenshot:
	go run $(IMGGEN_DIR)/*.go generate -tape $(IMGGEN_DIR)/tape/screenshot.tape -out $(OUTPUT_DIR)/screenshot
	
.PHONY: test
vrt:
	go run $(IMGDIFF_DIR)/*.go test -base ./img -target $(OUTPUT_DIR)/screenshot -out $(OUTPUT_DIR)/diff

.PHONY: clean
clean:
	rm -rf $(OUTPUT_DIR)
	