; The APACHE License (APACHE)
;
; Copyright (c) 2023 Lílian Ferreira de Freitas. All rights reserved.
;
; Licensed under the Apache License, Version 2.0 (the "License");
; you may not use this file except in compliance with the License.
; You may obtain a copy of the License at
;
;   http://www.apache.org/licenses/LICENSE-2.0
;
; Unless required by applicable law or agreed to in writing, software
; distributed under the License is distributed on an "AS IS" BASIS,
; WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
; See the License for the specific language governing permissions and
; limitations under the License.

; I could not find documentation for the device directive
; but to the best of my knowledge it hints the assembler
; to check for forbidden instructions for that device.
; http://ww1.microchip.com/downloads/en/DeviceDoc/AVR-InstructionSet-Manual-DS40002198.pdf#_OPENTOPIC_TOC_PROCESSING_d1951e73893
.device ATmega328P

; The define directive is specific to the AVRA preprocessor.
; https://github.com/Ro5bert/avra/blob/master/USAGE.md
; Memmory-mapped I/O addresses are in the manual:
; https://content.arduino.cc/assets/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf?_gl=1*1y8m4t*_ga*NTQ5Mjg5MDUuMTY3NDQxMTg4Mg..*_ga_NEXN8H46L5*MTY3NDQyNDk4NC4zLjEuMTY3NDQyNTUxOS4wLjAuMA..#G1446876
.define DDRB 0x04 ; I/O address (not RAM address which is 0x24)
.define PORTB 0x05 ; I/O address (not RAM address which is 0x25)
.define TCCR0A 0x24 ; Timer/Counter0 Control Register A
.define TCCR0B 0x25 ; Timer/Counter0 Control Register B
.define OCR0A  0x27 ; Timer/Counter0 Output Compare Register A
.define TIMSK0 0x6E ; Timer/Counter0 Interrupt Mask Register

.define TICKSL 0x00 ; data memory location for ticks low byte
.define TICKSH 0x01 ; data memory location for ticks high byte

; The origin directive sets the location counter for the
; current segment, in this case the Code segment.
; Set reset handler
.org 0x00 ; Reset vector (start of program on reset)
  jmp main ; jump changes the program counter to location of label main

; Set interrupt handlers to handle interrupts
.org 0x1C ; Timer/Counter0 compare match A Interrupt vector
  jmp count

; Interrupt handler for Timer/Counter0 compare match A IR
count:
  ; interrupts should come at 1 KHz
  ; count amount of interrupts and accumulate in TICKS
  push r16
  push r17
  
  lds r16, TICKSL
  lds r17, TICKSH
  
  inc r16
  brne end_count ; if did not overflow
  inc r17

end_count:
  sts TICKSL, r16
  sts TICKSH, r17

  pop r17
  pop r16

  reti

; sleep function, returns when 250ms have passed
sleep_500:
  push r16
  push r17
  push r26
  push r27
  
  ; load current value of ticks
  cli ; clear global interrupts
  lds r16, TICKSL
  lds r17, TICKSH
  sei ; set global interrupts

  ldi r26, 0xF4 ; load low byte of 500
  ldi r27, 0x01 ; load high byte of 500

  add r26, r16 ; add low byte
  adc r27, r17 ; add high byte with carry
  ; now r27:26 contains value of TICKS 500ms from now

check_500:
  ; load current value of ticks
  cli ; clear global interrupts
  lds r16, TICKSL
  lds r17, TICKSH
  sei ; set global interrupts

  ; compare r27:r26 with TICKS  
  cp r26, r16
  cpc r27, r17
  brne check_500

  pop r27
  pop r26
  pop r17
  pop r16
  ret

; sleep function, returns when 250ms have passed
sleep_250:
  push r16
  push r26

  ; load current value of ticks
  lds r16, TICKSL
  
  ldi r26, 250
  
  add r26, r16
  ; now r26 contains value of TICKS 250ms from now

check_250:
  ; load current value of ticks
  lds r16, TICKSL

  ; compare r26 with TICKS
  cp r26, r16
  brne check_250

  pop r26
  pop r16
  ret

; main function (entry point after reset as defined in .org)
main:
  ; Clear data at TICKS, the 16-bit counter we will use for time passed
  clr r16
  clr r17
  sts TICKSL, r16
  sts TICKSH, r17

  ; initialize timer
  call init_timer

  ; Set Data Register B5 to output (i.e. 1).
  sbi DDRB, 5 ; set bit 5 at address DDRB.

; infinite loop
loop:
  ; toggle LED connected to PORTB5 on for 500ms and off for 250ms
  sbi PORTB, 5 ; set bit 5 at address PORTB.
  call sleep_500
  cbi PORTB, 5 ; clear bit 5 at address PORTB.
  call sleep_250
  rjmp loop

init_timer:
  ; Timer/Counter Control Registers TCCR0A and TCCR0B
  ; control the Timer/Counter0 TCNT0.
  ; bits from 7 to 0
  ; TCCR0A 0x24 (0x44): COM0A1  COM0A0  COM0B1  COM0B0  –      –     WGM01  WGM00
  ; TCCR0B 0x25 (0x45): FOC0A   FOC0B   –       –       WGM02  CS02  CS01   CS00
  ; TCNT0  0x26 (0x46): Timer/Counter0 (8-bit)
  ; OCR0A  0x27 (0x47): Timer/Counter0 output compare register A
  ; OCR0B  0x28 (0x48): Timer/Counter0 output compare register B

  ; To set the timer we must use the Timer/Counter0 Control Registers A and B.
  ; We will construct the state of registers TCCR0A and TCCR0B in r17 and r18.
  clr r16 ; clear all bits on r18, will be state of TCCR0A
  clr r17 ; clear all bits on r17, will be state of TCCR0B

  ; Select a clocl with a prescaler of 64 (N=64) -> CS02:0 = 1
  ; CS02 CS01 CS00 (Clock Select0)
  ; 0    1    1
  ori r17, (1 << 0)|(1 << 1) ; set bit 0 and 1 on TCCR0B (CS00:1)
  
  ; Set the Timer/Counter0 to CTC Mode -> WGM02:0 = 2
  ; WGM02 WGM01 WGM00 (Waveform Generator0 Mode)
  ; 0     1     0
  ori r16, (1 << 1) ; set bit 1 of TCCR0A (WGM01)

  ;;; This is only needed if we want to generate a waveform
  ;;; output on OC0A. But we only need the interrupts now.
  ; Set the Compare Output Mode to Toggle Mode -> COM0A1:0 = 1
  ; This will toggle the value of OC0A (PORTD bit 6).
  ; ori r16, (1 << 6) ; set bit 6 of TCCR0A (COM0A0)
  
  out TCCR0A, r16 ; write to TCCR0A the state of r16
  out TCCR0B, r17 ; write to TCCR0B the state of r17
  clr r16
  clr r17

  ; Output Compare Register A controls the waveform frequency
  ; FREQ = CLOCK/(2*N*(1+OCR0A))
  ; we have CLOCK = 16 MHz (chip is on an Arduino Uno),
  ; we selected N = 64, so:
  ; if OCR0A = 249 -> FREQ = 500 Hz i.e. interrups come at 1000 Hz
  ldi r16, 249 ; load to r16 the desired value
  out OCR0A, r16 ; write to OCR0A
  clr r16

  ; Now with a frequency of 1 KHz the TCNT0 will
  ; reach the OCR0A value and set the corresponding
  ; flag (OCF0A) in the Timer/Counter0 Interrupt Flag Register (TIFR0)
  ; If the Timer/Counter0 Output Compare Match A Interrupt Enable (OCIE0A)
  ; is set, an Output Compare Interrupt will be generated
  ; and the flag will be cleared.
  ; Set OCIE0A, which is bit 1 of Timer/Counter Interrupt Mask Register (TIMSK0)
  ldi r16, (1 << 1) ; load bit 1 as set on r16
  sts TIMSK0, r16 ; store to TIMSK0
  clr r16

  ; set global interrupt enable bit
  sei

  ret
