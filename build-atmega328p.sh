#!/bin/sh
set -eu

cargo build --target=avr-specs/avr-atmega328p.json --release --no-default-features --features atmega328p
avr-objcopy -O ihex target/avr-attiny85/release/acurite-thermometer.elf target/avr-atmega328p/release/acurite-thermometer.hex
avr-objdump -d target/avr-atmega328p/release/acurite-thermometer.elf -l > target/avr-atmega328p/release/acurite-thermometer.S
avr-objdump -d --no-addresses --no-show-raw-insn target/avr-atmega328p/release/acurite-thermometer.elf -l > target/avr-atmega328p/release/acurite-thermometer.diff.S
