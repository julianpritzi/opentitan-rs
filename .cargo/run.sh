#!/bin/bash

PROJECT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )"/.. &> /dev/null && pwd )
BUILD_DIR="$PROJECT_DIR"/target/verilator/
OPENTITAN_PATH="${OPENTITAN_PATH:=$PROJECT_DIR/opentitan/}"

if [ -d "$BUILD_DIR" ]; then
    rm -R "$BUILD_DIR"
fi
mkdir "$BUILD_DIR"

riscv32-unknown-elf-objcopy ${1} "$BUILD_DIR"/opentitan-app.elf
riscv32-unknown-elf-objcopy --output-target=binary "$BUILD_DIR"/opentitan-app.elf "$BUILD_DIR"/opentitan-app.bin

srec_cat "$BUILD_DIR"/opentitan-app.bin\
		--binary --offset 0 --byte-swap 8 --fill 0xff \
		-within "$BUILD_DIR"/opentitan-app.bin\
		-binary -range-pad 8 --output "$BUILD_DIR"/opentitan-app.64.vmem --vmem 64

cd "$BUILD_DIR"

riscv32-unknown-elf-readelf -a opentitan-app.elf > opentitan-app.readelf
riscv32-unknown-elf-objdump -Cd opentitan-app.elf > opentitan-app.objdump

../../opentitan/bazel-out/k8-fastbuild/bin/hw/build.verilator_real/sim-verilator/Vchip_sim_tb \
		--meminit=rom,../../opentitan/bazel-out/k8-fastbuild-ST-2cc462681f62/bin/sw/device/lib/testing/test_rom/test_rom_sim_verilator.39.scr.vmem \
		--meminit=flash,./opentitan-app.64.vmem \
		--meminit=otp,../../opentitan/bazel-out/k8-fastbuild/bin/hw/ip/otp_ctrl/data/img_rma.24.vmem
