# Note on AVRA:
# avra ignores -f flag, even though it is shown
# in the command line options

# Makefile format:
# target: prerequisite
#   commands
# Pattern matching:
# % is widlcard
# $< matches prerequisite, $@ matches target
# See: https://makefiletutorial.com/

all: $(patsubst %.asm,%.hex,$(wildcard *.asm))

%.hex: %.asm
	@echo "Building..."
	avra $<
	@echo "\nCleaning unwanted files..."
	rm $(patsubst %.asm,%.obj,$<)
	rm $(patsubst %.asm,%.cof,$<)
	rm $(patsubst %.asm,%.eep.hex,$<)

clean:
	@echo "Removing .hex files..."
	rm -f $(patsubst %.asm,%.hex,$(wildcard *.asm))

upload: $(program).hex
	avrdude -p atmega328p -c arduino -P /dev/ttyACM0 -b 115200 -D -U flash:w:$<:i

monitor:
	picocom --send-cmd "ascii_xfr -s -v -l10" --nolock /dev/ttyACM0

.PHONY: all clean upload monitor
