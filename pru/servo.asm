;*
;* Copyright (C) 2016 Zubeen Tolani <ZeekHuge - zeekhuge@gmail.com>
;*
;* This file is as an example to show how to develope
;* and compile inline assembly code for PRUs
;*
;* This program is free software; you can redistribute it and/or modify
;* it under the terms of the GNU General Public License version 2 as
;* published by the Free Software Foundation.

;* This file has been modified in 2020 by T. Chamelot <tchamelot - chamelot.thomas@gmail.com>
;* The modifications imply
;* - Reducing the number of channel 4
;* - Modificating the timer mechanism in order to have a 20ms period (50Hz signal)
;*   - Use a register for the global period (20ms)
;*   - Use a register per channel as a timer (duty cycle)
;*   - Use memory shared with host to receive each channel command
;*   - Reset the register from the memory after each period
;* - Adding host / pru synchronisation for begining and ending

; PRU run at 200Mhz => in 20 ms there are 4e6 instructions
; This code takes:
; - 14 instructions per sample
; - 14 cycles to prepare the next cycle
; the number of sample for 20ms is
; (4e6 - 14) / 14 ~= 285713
NB_SAMPLE .set 285713

; An ESC expect an impulsion in the range [1-2]ms
; PRU run at 200Mhz => in 1ms there are 2e5 instructions
; The number of sample to get 1ms is
; 2e5 / 14 = 14285

; Most ESC controlled by a PWM required a wakeup signal.
; It is basically a signal with an impulsion of 0.9s
; 0.9 * 14285 = 12856
WAKEUP .set 12856

; Defintions of the channel used
    .asg	r30.t6,		CH4	; P8_39
    .asg	r30.t7,		CH3	; P8_40
    .asg	r30.t4,		CH2	; P8_41
    .asg	r30.t5,		CH1	; P8_42

; Channel command, address
SERVOMEM .set 0x0000

; PRU configuration
CPRUCFG .set c4

INTC    .set c0
SICR    .set 0x24
IRQ     .set 30
EVENT19 .set 19
EVENT21 .set 21

    .global _c_int00

_c_int00:
    lbco &r0, CPRUCFG, 4, 4             ; load SYSCFG
    clr r0, r0.t4                       ; clear SYSCFG[STANDBY_INIT]
    sbco &r0, CPRUCFG, 4, 4             ; enable OCP master port

    ;ldi 	r30, 0Xffff
	;delay 	10000000, R11
	;ldi		r30, 0X0000
	;delay 	10000000, R11

    ldi32 r1, WAKEUP                    ; Init channel with wakeup duty cycle
    ldi32 r2, WAKEUP
    ldi32 r3, WAKEUP
    ldi32 r4, WAKEUP
    ldi32 r28, SERVOMEM
    sbbo &r1, r28, 0, 4                 ; Save the current config into the shared memory 
    sbbo &r2, r28, 4, 4 
    sbbo &r3, r28, 8, 4 
    sbbo &r4, r28, 12, 4 
    ldi32 r29, NB_SAMPLE                ; Init the sample counter

    ldi r31.b0, 0x20 | (EVENT19 - 16)   ; notify begining

begin:
    qbeq clr1, r1, 0
    sub r1, r1, 1
    set r30, CH1
ch2:
    qbeq clr2, r2, 0
    sub r2, r2, 1
    set r30, CH2
ch3:
    qbeq clr3, r3, 0
    sub r3, r3, 1
    set r30, CH3
ch4:
    qbeq clr4, r4, 0
    sub r4, r4, 1
    set r30, CH4
sample_end:
    sub r29, r29, 1
    qbne begin, r29, 0

    lbbo &r1, r28, 0, 4                 ; take 2 + (len/4) cycle = 3 cycle
    lbbo &r2, r28, 4, 4                 ; take 2 + (len/4) cycle = 3 cycle
    lbbo &r3, r28, 8, 4                 ; take 2 + (len/4) cycle = 3 cycle
    lbbo &r4, r28, 12, 4                ; take 2 + (len/4) cycle = 3 cycle
    ldi32 r29, NB_SAMPLE
    qbbc begin, r31, IRQ                ; while host do not tell to stop
end:
    ldi r10, EVENT21
    sbco &r10, INTC, SICR, 2            ; clear event21
    clr r30, CH1                        ; clear channel 1
    clr r30, CH2                        ; clear channel 2
    clr r30, CH3                        ; clear channel 3
    clr r30, CH4                     ;   clear channel 4
    ldi r31.b0, 0x20 | (EVENT19 - 16)   ; notify host that it is finished
    halt                                ; Execution is over
    

clr1:
    clr r30, CH1
    qba ch2
clr2:
    clr r30, CH2
    qba ch3
clr3:
    clr r30, CH3
    qba ch4
clr4:
    clr r30, CH4
    qba sample_end
    
