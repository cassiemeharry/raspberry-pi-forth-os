SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.MAKEFLAGS += --warn-undefined-variables
.MAKEFLAGS += --no-builtin-rules

ifeq ($(origin .RECIPEPREFIX), undefined)
  $(error This Make does not support .RECIPEPREFIX. Please use GNU Make 4.0 or later)
endif
.RECIPEPREFIX = >

build: build/kernel8.img # build/kernel7.img
.PHONY: build

emulate-rpi3: build/kernel8.img
# > qemu-system-aarch64 -M raspi3 -kernel build/kernel8.elf -no-reboot -nographic -s &
> qemu-system-aarch64 -M raspi3 -device loader,file=build/kernel8.elf -device loader,addr=0x100000,cpu-num=0 -no-reboot -serial null -serial stdio -s &
> QEMU_PID=$$!
> gdb -ex 'target remote :1234' build/kernel8.elf
> kill $$QEMU_PID
# > qemu-system-aarch64 -M raspi3 -kernel build/kernel8.img -serial stdio
.PHONY: emulate-rpi3

# emulate-rpi2: build/kernel7.img
# > qemu-system-arm -kernel build/kernel7.img -M versatilepb -no-reboot -nographic
# # > qemu-system-aarch64 -M raspi2 -kernel build/kernel7.img -serial stdio
# .PHONY: emulate-rpi2

clean: clean-build
.PHONY: clean

clean-build:
> rm -r build/*
.PHONY: clean-build

clean-rust:
> rm -r target
.PHONY: clean-rust

clean-all: clean-build clean-rust
> rm -r target build/*
.PHONY: clean-all

build/kernel8.img: build/kernel8.elf
> aarch64-none-elf-objcopy build/kernel8.elf -O binary build/kernel8.img

# build/kernel7.img: build/kernel7.elf
# > aarch64-none-elf-objcopy build/kernel7.elf -O binary build/kernel7.img

build/kernel8.elf: build/boot.8.o build/utils.8.o build/kernel8.a src/linker-64.ld
> aarch64-none-elf-gcc -T src/linker-64.ld -o build/kernel8.elf -ffreestanding -O2 -nostdlib build/boot.8.o build/utils.8.o build/kernel8.a -lgcc

# build/kernel7.elf: build/boot.7.o build/utils.7.o build/kernel7.a src/linker-32.ld
# > arm-none-eabi-gcc -T src/linker-32.ld -o build/kernel7.elf -ffreestanding -O2 -nostdlib build/boot.7.o build/utils.7.o build/kernel7.a -lgcc

build/boot.8.o: src/boot.pi3-4.S src/exceptions.aarch64.S
> aarch64-none-elf-gcc -MMD -c src/boot.pi3-4.S -o build/boot.8.o

# build/boot.7.o: src/boot.pi2.S
# > arm-none-eabi-gcc -mcpu=cortex-a7 -fpic -ffreestanding -c src/boot.pi2.S -o build/boot.7.o

build/utils.8.o: src/utils.S
> aarch64-none-elf-gcc -MMD -c src/utils.S -o build/utils.8.o

# build/utils.7.o: src/utils.S
# > arm-none-eabi-gcc -mcpu=cortex-a7 -fpic -ffreestanding -c src/utils.S -o build/utils.7.o

build/kernel8.a: target/aarch64-unknown-none/debug/libraspberry_pi_forth_os.a
> cp target/aarch64-unknown-none/debug/libraspberry_pi_forth_os.a build/kernel8.a

# build/kernel7.a: target/armv7a-none-eabi/release/libraspberry_pi_forth_os.a
# > cp target/armv7a-none-eabi/release/libraspberry_pi_forth_os.a build/kernel7.a

target/aarch64-unknown-none/release/libraspberry_pi_forth_os.a: $(shell find src -type f -name '*.rs') Cargo.toml
> cargo xbuild --target=aarch64-unknown-none --release --no-default-features --features=rpi3

target/aarch64-unknown-none/debug/libraspberry_pi_forth_os.a: $(shell find src -type f -name '*.rs') Cargo.toml
> cargo xbuild --target=aarch64-unknown-none --no-default-features --features=rpi3

# target/armv7a-none-eabi/release/libraspberry_pi_forth_os.a: $(shell find src -type f -name '*.rs') Cargo.toml
# > cargo xbuild --target=armv7a-none-eabi --release --no-default-features --features=rpi2

# target/armv7a-none-eabi/debug/libraspberry_pi_forth_os.a: $(shell find src -type f -name '*.rs') Cargo.toml
# > cargo xbuild --target=armv7a-none-eabi --no-default-features --features=rpi2
