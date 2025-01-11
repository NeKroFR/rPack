# Compiler and flags
RUSTC=rustc
CARGO=cargo

# Output binary name
OUTPUT=rpack

# Source directories
SRC_DIR=src
CRYPTO_DIR=$(SRC_DIR)/crypto
MAIN_RS=$(SRC_DIR)/main.rs
SRC_FILES=$(MAIN_RS) $(wildcard $(CRYPTO_DIR)/*.rs)

# Default target: compile the binary
all: $(OUTPUT)

# Build the binary
$(OUTPUT): $(SRC_FILES)
	$(RUSTC) $(SRC_FILES) -o $(OUTPUT)

# Clean the build artifacts
clean:
	rm -f $(OUTPUT)

# Run the compiled binary
run: $(OUTPUT)
	./$(OUTPUT)

# Format the code using rustfmt
fmt:
	rustfmt $(SRC_FILES)

# Lint the code using clippy
lint:
	cargo clippy --all-targets --all-features

.PHONY: all clean run fmt lint

