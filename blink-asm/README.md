# Blink an LED / AVRA

Just blinking an LED using an 8-bit AVR microcontroller and assembly!

The code uses interrupts from the chip's Counter/Timer0 module for
implementing a "sleep" function which is just a busy loop waiting
on some specified amount of "ticks" to have passed.

## The hardware:

I'm using an Arduino Uno board, but this code mostly relies on two things:
- The microcontroller chip is the ATmega328P [[datasheet](https://content.arduino.cc/assets/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf)].
- The chip (ATmega328P) is clocked at 16 Mhz on the Arduino board.

From the Arduino Uno [datasheet](https://docs.arduino.cc/static/08d4f043936835a098b244c4714467c1/A000066-datasheet.pdf)
you can check that the builtin LED is connected to PB5, so this code sets
PORTB5 voltage to toggle that LED.

I use [`avrdude`](http://savannah.nongnu.org/projects/avrdude) to flash the chip.

To install in Debian: `sudo apt intall avrdude`.

## The code:

This code should be assembled with the AVR Assembler
[AVRA](https://github.com/Ro5bert/avra).

To install in Debian: `sudo apt install avra`.

The AVRA is mostly compatible with Atmel's own AVR Assembler, with a few
differences detailed in the README for the AVRA project, so Atmel's manuals
are a great resource:
- [AVR Assembler Manual](http://ww1.microchip.com/downloads/en/DeviceDoc/40001917A.pdf)
- [AVR Instruction Set Manual](http://ww1.microchip.com/downloads/en/DeviceDoc/AVR-InstructionSet-Manual-DS40002198.pdf)

## Makefile:

`make` (or `make blink.asm`) assembles the `blink.asm` file with `AVRA`,
then keeps only the Intel hex output, removing the others.

`make upload program=blink` uploads `blink.hex` to the board with `avrdude`,
assuming it is available at `/dev/ttyACM0`.

## Learning from:
-> https://gist.github.com/mhitza/8a4608f4dfdec20d3879
-> http://www.rjhcoding.com/avr-asm-getting-started-with-avra.php
