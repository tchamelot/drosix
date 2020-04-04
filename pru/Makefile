PRU_ASM = clpru
PRU_HEX = hexpru
ASM_FLAGS = --silicon_version=3

LINKER_CMD = servo.ld

DOCKER = docker
DOCKER_IMAGE = kylemanna/am335x

.PHONY: all
all: docker

docker:
	@$(DOCKER) run --rm --volume `pwd`:/work $(DOCKER_IMAGE) "make -C /work servo.bin"

%.bin: %.elf
	$(PRU_HEX) -b -o $@ $< --quiet

%.elf: %.obj $(LINKER_CMD)
	$(PRU_ASM) -z $(LINKER_CMD) -o $@ $<

%.obj: %.asm
	$(PRU_ASM) $(ASM_FLAGS) -c $<

