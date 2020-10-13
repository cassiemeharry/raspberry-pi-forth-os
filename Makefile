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

QEMU_FLAGS := -M raspi$(PI) -device loader,file=build/kernel8.elf
QEMU_FLAGS := $(QEMU_FLAGS) -no-reboot -serial null -serial stdio -d mmu,guest_errors,unimp
# QEMU_FLAGS := $(QEMU_FLAGS) -device dwc-usb2 -device usb-mouse -device usb-kbd

ifeq ($(shell if xset q 2>/dev/null; then echo 'yes'; else echo 'no'; fi),'no')
  QEMU_FLAGS := $(QEMU_FLAGS) -display none
endif

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

ENABLE_SEMIHOSTING ?= no
ifneq (no, $(ENABLE_SEMIHOSTING))
  # enable semihosting
  RUST_FEATURES := $(RUST_FEATURES) semihosting
endif

# Always enable semihosting in QEMU to exit VM.
QEMU_FLAGS := $(QEMU_FLAGS) -semihosting

ASM_FILES := $(wildcard src/asm/$(BIT_WIDTH)/*.S)
OBJ_FILES := $(ASM_FILES:src/asm/$(BIT_WIDTH)/%.S=build/%_s.$(ARM_VERSION).o)

DWC_OTG_C_FILES := $(wildcard vendor/dwc_otg/*.c)
OBJ_FILES := $(OBJ_FILES) $(DWC_OTG_C_FILES:vendor/dwc_otg/%.c=build/dwc_otg__%.$(ARM_VERSION).o)

DEP_FILES = $(OBJ_FILES:%.o=%.d)
-include $(DEP_FILES)

IMAGE_SIZE_MB = 5

SD_CARD_SOURCES := boot/bootcode.bin boot/config.txt build/kernel$(ARM_VERSION).img boot/start.elf
SD_CARD_FILES := bootcode.bin config.txt kernel$(ARM_VERSION).img start.elf

build: build/sdcard.$(ARM_VERSION).img tftp

build-rust: target/$(RUST_TRIPLE)/$(RUST_OPT_LEVEL)/libraspberry_pi_forth_os.a

emulate-rpi3: build/kernel8.img
> qemu-system-aarch64 $(QEMU_FLAGS)
# > qemu-system-aarch64 -M raspi3 -semihosting build/kernel8.img -no-reboot -serial null -serial stdio -d mmu,guest_errors
.PHONY: emulate-rpi3

debug-rpi3: build/kernel8.img build/kernel8.elf
> qemu-system-aarch64 $(QEMU_FLAGS) -S -s &
> QEMU_PID=$$! # -ex 'layout split'
> gdb -ex 'target remote :1234' -ex 'display /3i $$pc' -ex 'symbol-file build/kernel8.elf' # -o -0xFFFF000000000000'
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

tftp: build/tftp $()

build/tftp: $(SD_CARD_FILES:%=build/sdcard.$(ARM_VERSION)/%)
> if [[ ! -L "$@" ]]; then
>   ln -s /srv/tftp "$@"
> fi
> cp $^ "$@"

build/sdcard.$(ARM_VERSION).img: build/sdcard.$(ARM_VERSION).fat boot/sfdisk.txt
> dd if=/dev/zero of="$@" count=$(IMAGE_SIZE_MB) bs=1M
> sfdisk --no-tell-kernel "$@" < boot/sfdisk.txt
> dd if=build/sdcard.$(ARM_VERSION).fat of="$@" seek=1 bs=1M
> sfdisk --verify "$@" < boot/sfdisk.txt
> parted "$@" -- print

build/sdcard.$(ARM_VERSION).fat: $(SD_CARD_FILES:%=build/sdcard.$(ARM_VERSION)/%)
> set -x
> dd if=/dev/zero of="$@" count=$$(( $(IMAGE_SIZE_MB) - 1 )) bs=1M
> mkfs.vfat -v -n 'forth-os' -F 16 -S 512 -s 1 "$@"
> for file in $^; do
>   mcopy -i "$@" "$$file" '::'"$$(basename "$$file")"
> done
> mdir -i "$@" '::'
> ls -l "$@"

build/sdcard.$(ARM_VERSION)/kernel$(ARM_VERSION).img: build/kernel$(ARM_VERSION).img
> cp $< "$@"

build/sdcard.$(ARM_VERSION)/%: boot/% build/sdcard.$(ARM_VERSION)
> cp $< "$@"

build/sdcard.$(ARM_VERSION):
> mkdir -p "$@"

build/kernel$(ARM_VERSION).img: build/kernel$(ARM_VERSION).elf
> $(ARM_PREFIX)-objcopy $^ -O binary $@

build/kernel$(ARM_VERSION).elf: src/linker-64.ld $(OBJ_FILES) build/kernel$(ARM_VERSION).a
> $(ARM_PREFIX)-gcc -T $< -o $@ -ffreestanding -O2 -nostdlib -lgcc $^

build/%_s.8.o: src/asm/64/%.S src/asm/64/%.h
> $(ARM_PREFIX)-gcc -MMD -c $< -o $@

build/%_s.7.o: src/asm/32/%.S src/asm/32/%.h
> $(ARM_PREFIX)-gcc -mcpu=cortex-a7 -fpic -ffreestanding -c $< -o $@

build/dwc_otg__%.8.o: vendor/dwc_otg/%.c
> $(ARM_PREFIX)-gcc -MMD -O2 -DCONFIG_USB_DWC_OTG -DCONFIG_USB_KEYBOARD -Wno-address-of-packed-member -Wno-pointer-to-int-cast -Wno-int-to-pointer-cast -c $< -o $@

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

target/$(RUST_TRIPLE)/$(RUST_OPT_LEVEL)/libraspberry_pi_forth_os.a: $(shell find src -type f -name '*.rs') Cargo.toml .cargo/config Makefile
> cargo xbuild --target=$(RUST_TRIPLE) $(RUST_RELEASE_FLAG) --features=$(RUST_FEATURES_FLAG)
> touch "$@"
