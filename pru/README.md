# PRU sources for Drosix

This folder contains the source code of the PRU firmware for Drosix.

## Requirements

The Makefile use Docker in order to cross compile the firmware. If you are 
planning to build the firmware directly on the Beaglebone or you have `clpru` 
already installed you can skip the requirements and follow the alternative build 
method.

1. Install [Docker](https://docs.docker.com/install/)
2. Install [kylemanna/am335x](https://hub.docker.com/r/kylemanna/am335x/) image 
   with `docker pull kylemanna/am335x`

## Build

If you have followed the requirement, you only need to run `make all`.

Alternatively, it is possible to build the firmware directly by running 
`make servo.bin`.

## ESC firmware

The ESC firmware allows to control 4 ESCs from the PRU1. The firmware is self 
explained in the source code. Do not fear assembly code. It communicates with 
the host through shared memory at the address 0x2000. The memory is encoded in 
little endian. The variables control for each ESC the number of sample during 
the signal should be up. This number should be in the range [12856;28570] which 
map to impulsions in range [0.9;2]ms.

| Address | Variable |
| ------- | -------- |
| 0x2000  | esc1     |
| 0x2004  | esc2     |
| 0x2008  | esc3     |
| 0x200c  | esc4     |
