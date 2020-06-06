CC = clpru
OBJCOPY = hexpru

ASM_FLAGS := --silicon_version=3
ASFLAGS := -pden
CFLAGS := 	-Ipru-software-support-package/include -I$(PRU_CGT)/include \
			--hardware_mac=on -O2 \
			-pden -pdew -pdr --check_misra=all -q
LDFLAGS := -z am335x.ld -q -pdew -i$(PRU_CGT)/lib --library=libc.a

LINKER_CMD = servo.ld

DOCKER = docker
DOCKER_IMAGE = kylemanna/am335x

.PHONY: all build clean docker docker-interactive
all: docker

build: servo.bin controller.bin

docker:
	@$(DOCKER) run --rm --volume `pwd`:/work $(DOCKER_IMAGE) "make -C /work build"

docker-interactive:
	@$(DOCKER) run -it --rm --volume `pwd`:/work $(DOCKER_IMAGE) "bash"

%.bin: %.elf
	@$(OBJCOPY) -b -o $@ $< --quiet

%.elf: %.obj
	@$(CC) $^ $(LDFLAGS) -o $@

%.obj: %.asm
	@$(CC) $(ASFLAGS) -c $<

%.obj: %.c drosix.h util.h
	@$(CC) $(CFLAGS) -c $<

clean:
	@rm -rf *.obj *.bin
