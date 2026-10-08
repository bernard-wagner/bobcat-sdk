
.PHONY: clean build

OBJ_DIR := target/wasm32-unknown-unknown/release

DIRS := \
	001-counter \
	002-mul_div \
	003-camelot-swapping \
	005-counter-bobcat-return \
	006-supercats

OBJ_COUNTER := ${OBJ_DIR}/bobcat_sdk_examples_001.wasm
OBJ_MUL_DIV := ${OBJ_DIR}/bobcat-sdk-examples-002.wasm
OBJ_CAMELOT := ${OBJ_DIR}/bobcat-sdk-examples-003.wasm
OBJ_COUNTER_BOBCAT_RETURN := ${OBJ_DIR}/bobcat_sdk_examples_005.wasm
OBJ_SUPERCATS := ${OBJ_DIR}/bobcat-sdk-examples-006.wasm

build: \
	001-counter.wasm \
	002-mul_div.wasm \
	003-camelot.wasm \
	005-counter-bobcat-return.wasm \
	006-supercats.wasm

${OBJ_COUNTER}: $(shell find Cargo.* ${DIRS} -type f -name Cargo.toml -or -name '*.rs')
	@cargo build --target wasm32-unknown-unknown --release

001-counter.wasm: ${OBJ_COUNTER}
	@./wasm-post.sh ${OBJ_COUNTER} 001-counter.wasm

002-mul_div.wasm: ${OBJ_COUNTER}
	@./wasm-post.sh ${OBJ_MUL_DIV} 002-mul_div.wasm

003-camelot.wasm: ${OBJ_COUNTER}
	@./wasm-post.sh ${OBJ_CAMELOT} 003-camelot.wasm

005-counter-bobcat-return.wasm: ${OBJ_COUNTER}
	@./wasm-post.sh \
		${OBJ_COUNTER_BOBCAT_RETURN} \
		005-counter-bobcat-return.wasm

006-supercats.wasm: ${OBJ_COUNTER}
	@./wasm-post.sh \
		${OBJ_SUPERCATS} \
		006-supercats.wasm

clean:
	@cargo clean
	@rm -f 001-counter.wasm 002-mul_div.wasm 003-camelot.wasm 005-counter-bobcat-return.wasm 006-supercats.wasm