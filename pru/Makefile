CC = clpru
OBJCOPY = hexpru

PRU_CGT := /usr/share/ti-cgt-pru_2.3.3
PRU_SUPPORT_PACKAGE := /usr/share/pru-software-support-package

ASM_FLAGS := --silicon_version=3
ASFLAGS := -pden
CFLAGS := 	-I$(PRU_SUPPORT_PACKAGE)/include -I$(PRU_CGT)/include \
			--hardware_mac=on -O2 \
			-pden -pdew -pdr -q \
			--obj_directory=build --obj_extension=.o # --check_misra=all -q
LDFLAGS := -z am335x.ld -q -pdew -i$(PRU_CGT)/lib --library=libc.a

LINKER_CMD = servo.ld

TARGETS := build/controller.bin build/motor.bin

CONTROLLER_SRC = src/controller.c src/pid.c
MOTOR_SRC = src/motor.c

CONTROLLER_OBJ = $(patsubst src/%.c, build/%.o, $(CONTROLLER_SRC))
MOTOR_OBJ = $(patsubst src/%.c, build/%.o, $(MOTOR_SRC))

.PHONY: all build clean

all: $(TARGETS)

build/controller.elf: $(CONTROLLER_OBJ)
	@echo "(LD) $@"
	@$(CC) $^ $(LDFLAGS) -o $@

build/motor.elf: $(MOTOR_OBJ)
	@echo "(LD) $@"
	@$(CC) $^ $(LDFLAGS) -o $@

%.bin: %.elf
	@$(OBJCOPY) -b -o $@ $< --quiet

%.elf: %.o
	@echo "(LD) $@"
	@$(CC) $^ $(LDFLAGS) -o $@

%.o: %.asm
	@$(CC) $(ASFLAGS) -c $<

build/%.o: src/%.c src/drosix.h src/util.h
	@echo "(CC) $@"
	@$(CC) $(CFLAGS) -c $<

clean:
	@rm -rf build/*
