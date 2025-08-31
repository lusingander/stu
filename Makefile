# Note:
#   This Makefile is only used for generating screenshots and testing them,
#   it is not related to the development of the application itself.

PROJECT_DIR := $(shell pwd)

STU_BIN=$(PROJECT_DIR)/target/debug/stu
RUST_SRC=$(PROJECT_DIR)/src/*.rs
CMD_DIR=$(PROJECT_DIR)/tool
STU_ROOT_DIR=$(PROJECT_DIR)/tool/imggen/test_root_dir
IMGGEN_DIR=$(CMD_DIR)/imggen
IMGDIFF_DIR=$(CMD_DIR)/imgdiff
OUTPUT_DIR=$(PROJECT_DIR)/out
IMG_DIR=$(PROJECT_DIR)/img

$(STU_BIN): $(RUST_SRC)
	cargo build

.PHONY: demo
demo: $(STU_BIN)
	cd $(IMGGEN_DIR) && go run *.go generate \
		-tape $(IMGGEN_DIR)/tape/demo.tape \
		-bin $(STU_BIN) \
		-root $(STU_ROOT_DIR) \
		-out $(OUTPUT_DIR)/demo

.PHONY: social-preview-demo
social-preview-demo: $(STU_BIN)
	cd $(IMGGEN_DIR) && go run *.go generate \
		-tape $(IMGGEN_DIR)/tape/social-preview-demo.tape \
		-bin $(STU_BIN) \
		-root $(STU_ROOT_DIR) \
		-out $(OUTPUT_DIR)/social-preview-demo

.PHONY: screenshot
screenshot: $(STU_BIN)
	cd $(IMGGEN_DIR) && go run *.go generate \
		-tape $(IMGGEN_DIR)/tape/screenshot.tape \
		-bin $(STU_BIN) \
		-root $(STU_ROOT_DIR) \
		-out $(OUTPUT_DIR)/screenshot

.PHONY: vrt
vrt: screenshot
	cd $(IMGDIFF_DIR) && go run *.go test \
		-base $(IMG_DIR) \
		-target $(OUTPUT_DIR)/screenshot \
		-out $(OUTPUT_DIR)/diff

.PHONY: update-img
update-img:
	mv $(OUTPUT_DIR)/screenshot/*.png $(IMG_DIR)

.PHONY: clean
clean:
	rm -rf $(OUTPUT_DIR)
	