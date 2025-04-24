STU_BIN=./target/debug/stu
RUST_SRC=./src/*.rs
CMD_DIR=./tool
IMGGEN_DIR=$(CMD_DIR)/imggen
IMGDIFF_DIR=$(CMD_DIR)/imgdiff
OUTPUT_DIR=./out
CARGO_BUILD_FLAGS?=--features imggen
RELEASE?=0
ifneq ($(RELEASE), 0)
	CARGO_BUILD_FLAGS += --release
	STU_BIN=./target/release/stu
endif

$(STU_BIN): $(RUST_SRC)
	cargo build $(CARGO_BUILD_FLAGS)

.PHONY: demo
demo: $(STU_BIN)
	go run $(IMGGEN_DIR)/*.go generate -tape $(IMGGEN_DIR)/tape/demo.tape -out $(OUTPUT_DIR)/demo

.PHONY: social-preview-demo
social-preview-demo: $(STU_BIN)
	go run $(IMGGEN_DIR)/*.go generate -tape $(IMGGEN_DIR)/tape/social-preview-demo.tape -out $(OUTPUT_DIR)/social-preview-demo

.PHONY: screenshot
screenshot: $(STU_BIN)
	go run $(IMGGEN_DIR)/*.go generate -tape $(IMGGEN_DIR)/tape/screenshot.tape -out $(OUTPUT_DIR)/screenshot
	
.PHONY: vrt
vrt: screenshot
	go run $(IMGDIFF_DIR)/*.go test -base ./img -target $(OUTPUT_DIR)/screenshot -out $(OUTPUT_DIR)/diff

.PHONY: clean
clean:
	rm -rf $(OUTPUT_DIR)
	rm -rf target
