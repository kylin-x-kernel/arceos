# pKVM configuration
PKVM_SIGN_TOOL := $(PKVM_TOOL_PATH)/sign-tools
PKVM_TMP_DIR   := $(PKVM_SIGN_TOOL)/tmp
OUT_PKVM_BIN   := $(patsubst %.elf,%.pkvm.bin,$(OUT_ELF))
WORKSPACE_DIR  := $(shell pwd)

# pKVM VM configuration
PKVM_CPUS      := 2
PKVM_MEM_SIZE  := 2048
PKVM_VSOCK_CID := 3

define run_pkvm
	@echo "==> Checking dependencies..."
	@if [ -z "$$(gmssl version 2>/dev/null)" ]; then \
		echo "Error: GMSSL not found. Please install GMSSL v3.0.0 or later from https://gmssl.org/"; \
		exit 1; \
	fi
	@if [ ! -f "$(PKVM_TOOL_PATH)/pvm-manage" ]; then \
		echo "Error: pvm-manage not found at $(PKVM_TOOL_PATH)/pvm-manage"; \
		exit 1; \
	fi

	@echo "==> Generating digital signature for kernel..."
	@cp $(OUT_BIN) $(PKVM_TMP_DIR)/kernel.bin
	@cd $(PKVM_SIGN_TOOL) && ./gm_sign_guest_kernel_only.sh
	@mv $(PKVM_TMP_DIR)/kernel.bin $(OUT_PKVM_BIN)

	@echo "==> Signed kernel: $(OUT_PKVM_BIN) $(PKVM_TOOL_PATH)"
	@echo "==> Verifying disk image..."
	@if [ ! -f "$(WORKSPACE_DIR)/$(DISK_IMG)" ]; then \
		echo "Error: Disk image $(WORKSPACE_DIR)/$(DISK_IMG) does not exist"; \
		exit 1; \
	fi

	@echo "==> Launching protected VM..."
	@cd $(PKVM_TOOL_PATH) && sudo ./pvm-manage cleanall && \
    sudo ./pvm-manage run --protected-vm -- \
		--cpus $(PKVM_CPUS) \
		--mem size=$(PKVM_MEM_SIZE) \
		--block $(WORKSPACE_DIR)/$(DISK_IMG) \
		--vsock $(PKVM_VSOCK_CID) \
		$(OUT_PKVM_BIN)
endef