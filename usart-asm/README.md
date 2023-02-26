# USART / ASM

Sending serial data through USART on an 8-bit
AVR microcontroller with assembly!

The program sends "Hello there!" (over and over again!)
through the USART0, it can be seen with a serial monitor
such as [`picocom`](https://linux.die.net/man/8/picocom).

## The hardware:

I'm using an Arduino Uno board, but this code mostly relies on two things:
- The microcontroller chip is the ATmega328P [[datasheet](https://content.arduino.cc/assets/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf)].
- The chip (ATmega328P) is clocked at 16 Mhz on the Arduino board.

You may want to check the Arduino Uno [datasheet here](https://docs.arduino.cc/static/08d4f043936835a098b244c4714467c1/A000066-datasheet.pdf).

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

`make` (or `make usart.asm`) assembles the `usart.asm` file with `AVRA`,
then keeps only the Intel hex output, removing the others.

`make upload program=usart` uploads `usart.hex` to the board with `avrdude`,
assuming it is available at `/dev/ttyACM0`.

`make monitor` uses `picocom` to monitor the serial data transmitted
by the board. To exit use ctrl+a ctrl+x.

## Learning from:
- https://gist.github.com/mhitza/8a4608f4dfdec20d3879
- http://www.rjhcoding.com/avr-asm-getting-started-with-avra.php
- http://www.rjhcoding.com/avr-asm-uart.php
- http://www.rjhcoding.com/avr-asm-pm.php
- https://hekilledmywire.wordpress.com/2011/01/05/using-the-usartserial-tutorial-part-2/
