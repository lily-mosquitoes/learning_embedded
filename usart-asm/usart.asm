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

.device ATmega328P

; Timer
.define TCCR0A 0x24 ; Timer/Counter0 Control Register A
.define TCCR0B 0x25 ; Timer/Counter0 Control Register B
.define OCR0A  0x27 ; Timer/Counter0 Output Compare Register A
.define TIMSK0 0x6E ; Timer/Counter0 Interrupt Mask Register
; Ticks - on data space
.define TICKSL 0x00
.define TICKSH 0x01

; USART
.define UDR0 0xC6 ; USART0 data register
.define UBRR0H 0xC5 ; Baud rate scaler high bit 
.define UBRR0L 0xC4 ; Baud rate scaler low bit
.define UCSR0C 0xC2 ; USART0 control and status register C
.define UCSR0B 0xC1 ; USART0 control and status register B
.define UCSR0A 0xC0 ; USART0 control and status register A

; Reset handler
.org 0x00
  jmp main

; Timer/Counter0 Compare Match A Interrupt handler
.org 0x1C
  jmp count

count:
  ; interrupts should come at 1 KHz
  ; count amount of interrupts and accumulate in TICKS
  push r16
  push r17

  lds r16, TICKSL
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
  ; Clear data at TICKS, the 8-bit counter we will use for time passed
  clr r16
  sts TICKSH, r16
  sts TICKSL, r16

  ; initialize timer
  call init_timer

  ; setup USART
  call setup_usart

; string to transmit, null terminated - on program memory
; space allocated in Program Memory must be an even number of bytes.
str: .db "Hello there!!", 0x0D, 0x0A, 0x00

; infinite loop
loop:
  rcall send_string
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

  ; Output Compare Register A controls the interrupt frequency
  ; FREQ = CLOCK/(N*(1+OCR0A))
  ; we have CLOCK = 16 MHz (chip is on an Arduino Uno),
  ; we selected N = 64, so:
  ; if OCR0A = 249 -> FREQ = 1000 Hz
  ; i.e. interrups come at 1000 per secont
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

send_string:
  ; initiate the Z pointer to load str from program data
  ; Z pointer is byte addressed but program data is word
  ; addressed, so Z pointer addr is 2*prog data addr for
  ; lower byte and 2*prog data addr + 1 for higher byte
  ; low and high are used for simplification
  ldi r30, low(2*str)
  ldi r31, high(2*str)

load_byte:
  lpm r16, Z+
  cpi r16, 0x00 ; compare with null char
  breq end_string ; branch if equal

  rcall send_byte
  rjmp load_byte

end_string:
  ret

send_byte:
  ; Obs:
  ; When the transmitter is enabled, the normal port operation
  ; of the TxDn pin is overridden by the USART and given the
  ; function as the transmitter’s serial output.
  ;
  ; A data transmission is initiated by loading the transmit
  ; buffer with the data to be transmitted. The CPU can load
  ; the transmit buffer by writing to the UDRn I/O location.
  ;
  ; Protocol:
  ; 1. poll the data register to assure it is empty
  ;    UDREn flag = bit 5 of UCSR0A should be set if
  ;    USART is ready to transmit
  ; 2. if empty, write data to the register (UDRn)
  lds r17, UCSR0A
  sbrs r17, 5 ; skip if UDRE0 bit is set
  rjmp send_byte
  ; send byte on r16
  sts UDR0, r16
  ret

setup_usart:
  push r16
  push r17
  ; UMSELn bit in USART control and status register C
  ; (UCSRnC) selects between asynchronous and synchronous operation.
  ; The baud rate generator clock output = CLOCK/(UBRRn+1).
  ; The transmitter divides the baud rate generator clock
  ; output by 2, 8 or 16 depending on mode.
  ; Asynchronous normal mode (U2Xn = 0)
  ; Equation for Calculating UBRRn Value:
  ; UBRRn = (CLOCK/16BAUD)-1
  ; BAUD -> Baud rate (in bits per second, bps)
  ; UBRRn -> Contents of the UBRRnH and UBRRnL registers, (0-4095)
  ; recommended baud rate -> 9600 bits/s
  ; so UBRRn should be ~ 103 -> goes on the UBRRnH and UBRRnL
  ldi r16, 0x00
  ldi r17, 0x67
  sts UBRR0H, r16
  sts UBRR0L, r17
  ; The frame format used by the USART is set by the UCSZn2:0,
  ; UPMn1:0 and USBSn bits in UCSRnB and UCSRnC.
  ; UCSRnB – USART Control and Status Register n B:
  ; Bit 3 – TXENn: Transmitter Enable n
  ldi r16, (1 << 3)
  sts UCSR0B, r16
  ; UCSRnC – USART Control and Status Register n C
  ; Bits 7:6 - UMSELn1:0 USART Mode Select
  ; 00 -> Asynchronous USART (default)
  ; Bit 3 – USBSn: Stop Bit Select
  ; 1 -> 2 bit
  ; Bit 2:1 – UCSZn1:0: Character Size
  ; 11 -> 8 bits
  ldi r16, (7 << 1)
  sts UCSR0C, r16

  pop r17
  pop r16
  ret
