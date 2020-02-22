SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.MAKEFLAGS += --warn-undefined-variables
.MAKEFLAGS += --no-builtin-rules

ifeq ($(origin .RECIPEPREFIX), undefined)
  $(error This Make does not support .RECIPEPREFIX. Please use GNU Make 4.0 or later)
endif
.RECIPEPREFIX = >

PI := 3

ifeq (3, $(PI))
  ARM_PREFIX = aarch64-none-elf
  ARM_VERSION = 8
  BIT_WIDTH = 64
  RUST_TRIPLE = aarch64-unknown-none
else ifeq (2, $(PI))
  ARM_PREFIX = arm-none-eabi
  ARM_VERSION = 7
  BIT_WIDTH = 64
  RUST_TRIPLE = armv7a-none-eabi
else
  $(error Variable PI must be either 2 or 3, found $(PI))
endif

RUST_OPT_LEVEL := release
RUST_FEATURES := rpi$(PI)
# enable semihosting
RUST_FEATURES := $(RUST_FEATURES) semihosting

ASM_FILES := $(wildcard src/asm/$(BIT_WIDTH)/*.S)
OBJ_FILES := $(ASM_FILES:src/asm/$(BIT_WIDTH)/%.S=build/%_s.$(ARM_VERSION).o)

# DWC_OTG_C_FILES := $(wildcard vendor/dwc_otg/*.c)
# OBJ_FILES := $(OBJ_FILES) $(DWC_OTG_C_FILES:vendor/dwc_otg/%.c=build/dwc_otg__%.$(ARM_VERSION).o)

DEP_FILES = $(OBJ_FILES:%.o=%.d)
-include $(DEP_FILES)

build: build/kernel$(ARM_VERSION).img

emulate-rpi3: build/kernel8.img
> qemu-system-aarch64 -M raspi3 -semihosting -device loader,file=build/kernel8.elf -no-reboot -serial null -serial stdio -d mmu,guest_errors
.PHONY: emulate-rpi3

debug-rpi3: build/kernel8.img build/kernel8.elf
> qemu-system-aarch64 -M raspi3 -semihosting -device loader,file=build/kernel8.img -no-reboot -serial null -serial stdio -d mmu,guest_errors -S -s &
> QEMU_PID=$$! # -ex 'layout split'
> gdb -ex 'target remote :1234' -ex 'display /3i $$pc' -ex 'symbol-file build/kernel8.elf' #  -o -0xFFFF000000000000
> kill $$QEMU_PID
# > qemu-system-aarch64 -M raspi3 -kernel build/kernel8.img -serial stdio
.PHONY: debug-rpi3

# emulate-rpi2: build/kernel7.img
# > qemu-system-arm -kernel build/kernel7.img -M versatilepb -no-reboot -nographic
# # > qemu-system-aarch64 -M raspi2 -kernel build/kernel7.img -serial stdio
# .PHONY: emulate-rpi2

clean: clean-build
.PHONY: clean

clean-build:
> rm -rf build/*
.PHONY: clean-build

clean-rust:
> rm -r target
.PHONY: clean-rust

clean-all: clean-build clean-rust
> rm -r target build/*
.PHONY: clean-all

build/kernel$(ARM_VERSION).img: build/kernel$(ARM_VERSION).elf
> $(ARM_PREFIX)-objcopy $^ -O binary --image-base=0x80000 $@

build/kernel$(ARM_VERSION).elf: src/linker-64.ld $(OBJ_FILES) build/kernel$(ARM_VERSION).a
> $(ARM_PREFIX)-gcc -T $< -o $@ -ffreestanding -O2 -nostdlib -lgcc $^

build/%_s.8.o: src/asm/64/%.S src/asm/64/%.h
> $(ARM_PREFIX)-gcc -MMD -DENABLE_SEMIHOSTING -c $< -o $@

build/%_s.7.o: src/asm/32/%.S src/asm/32/%.h
> $(ARM_PREFIX)-gcc -mcpu=cortex-a7 -fpic -ffreestanding -c $< -o $@

# build/dwc_otg__%.8.o: vendor/dwc_otg/%.c
# > $(ARM_PREFIX)-gcc -MMD -DENABLE_SEMIHOSTING -DCONFIG_USB_DWC_OTG -c $< -o $@

build/kernel$(ARM_VERSION).a: target/$(RUST_TRIPLE)/$(RUST_OPT_LEVEL)/libraspberry_pi_forth_os.a
> cp "$^" "$@"

ifeq ($(RUST_OPT_LEVEL), release)
  RUST_RELEASE_FLAG = --release
else
  RUST_RELEASE_FLAG =
endif

comma := ,
empty :=
space := $(empty) $(empty)
RUST_FEATURES_FLAG := $(subst $(space),$(comma),$(RUST_FEATURES))

target/$(RUST_TRIPLE)/$(RUST_OPT_LEVEL)/libraspberry_pi_forth_os.a: $(shell find src -type f -name '*.rs') Cargo.toml .cargo/config
> cargo xbuild --target=$(RUST_TRIPLE) $(RUST_RELEASE_FLAG) --no-default-features --features=$(RUST_FEATURES_FLAG)
> touch "$@"
