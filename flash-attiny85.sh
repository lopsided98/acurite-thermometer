#!/bin/sh
set -eu

cargo build --target=avr-specs/avr-attiny85.json --release --no-default-features --features attiny85
avr-objcopy -O ihex target/avr-attiny85/release/acurite-thermometer.elf target/avr-attiny85/release/acurite-thermometer.hex
avr-objdump -d target/avr-attiny85/release/acurite-thermometer.elf -l > target/avr-attiny85/release/acurite-thermometer.S
avr-objdump -d --no-addresses --no-show-raw-insn target/avr-attiny85/release/acurite-thermometer.elf -l > target/avr-attiny85/release/acurite-thermometer.diff.S
avrdude -p attiny85 -c avrisp -b 19200 -P /dev/ttyACM0 -U flash:w:target/avr-attiny85/release/acurite-thermometer.hex