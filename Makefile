.PHONY: test testgdb tri

# DEBUG_LEVEL=all
DEBUG_LEVEL=warning,error

PARAMS=VK_ICD_FILENAMES=$(PWD)/softvk.json LD_LIBRARY_PATH=$(PWD)/sdk VK_LOADER_DEBUG=$(DEBUG_LEVEL) RUST_LOG=softvk

test:
	$(PARAMS) sdk/vulkaninfo

testgdb:
	$(PARAMS) gdb --args sdk/vulkaninfo

tri:
	$(PARAMS) sdk/tri

trigdb:
	$(PARAMS) gdb --args sdk/tri
