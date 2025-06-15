# Note:
#   This Makefile is only used for generating screenshots and testing them,
#   it is not related to the development of the application itself.

STU_BIN=./target/debug/stu
RUST_SRC=./src/*.rs
CMD_DIR=./tool
STU_ROOT_DIR=./tool/imggen/test_root_dir
IMGGEN_DIR=$(CMD_DIR)/imggen
IMGDIFF_DIR=$(CMD_DIR)/imgdiff
OUTPUT_DIR=./out

$(STU_BIN): $(RUST_SRC)
	cargo build --features imggen

.PHONY: demo
demo: $(STU_BIN)
	go run $(IMGGEN_DIR)/*.go generate -tape $(IMGGEN_DIR)/tape/demo.tape -bin $(STU_BIN) -root $(STU_ROOT_DIR) -out $(OUTPUT_DIR)/demo

.PHONY: social-preview-demo
social-preview-demo: $(STU_BIN)
	go run $(IMGGEN_DIR)/*.go generate -tape $(IMGGEN_DIR)/tape/social-preview-demo.tape -bin $(STU_BIN) -root $(STU_ROOT_DIR) -out $(OUTPUT_DIR)/social-preview-demo

.PHONY: screenshot
screenshot: $(STU_BIN)
	go run $(IMGGEN_DIR)/*.go generate -tape $(IMGGEN_DIR)/tape/screenshot.tape -bin $(STU_BIN) -root $(STU_ROOT_DIR) -out $(OUTPUT_DIR)/screenshot
	
.PHONY: vrt
vrt: screenshot
	go run $(IMGDIFF_DIR)/*.go test -base ./img -target $(OUTPUT_DIR)/screenshot -out $(OUTPUT_DIR)/diff

.PHONY: clean
clean:
	rm -rf $(OUTPUT_DIR)
	