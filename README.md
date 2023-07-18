# Acurite Thermometer Firmware

Firmware for a custom wireless thermometer that speaks the protocol used by the Acurite 00606TX. This firmware is based around an ATtiny85, TMP102 temperature sensor and a 433.92 MHz OOK radio such as one extracted from the original 00606TX hardware. The firmware is written in Rust using [avr-hal](https://github.com/Rahix/avr-hal).

I have an Acurite 00606TX wireless thermometer that started reporting incorrect temperatures. Rather than buy a new one, I decided to replace its internals. The radio (and LED) can be easily removed from the original PCB, as it is only attached by two tabs. The radio PCB is well labeled, and even contains unpopulated through-holes for the data and ground connections.

The radio protocol is described in detail here: https://wiki.jmehan.com/display/KNOW/Reverse+Engineering+Acurite+Temperature+Sensor. Additionally, the transmitter must transmit at reasonably precise 31 second intervals, as the receiver only listens for a short period every 31 seconds. This firmware uses the 128 kHz watchdog clock for timing, which may not be sufficiently accurate across the full temperature and voltage range.