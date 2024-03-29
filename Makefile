DOCKER := docker
DOCKER_BUILD_STAMP := .stamp_docker
DOCKER_IMAGE := drosix_dev
DOCKER_NAME := drosix_dev
DROSIX_DL := drosix_dl
DROSIX_OUTPUT := drosix_output
CARGO_REGISTRY := drosix_cargo
GENERATR := /home/tchamelot/workspace/structurizr-site-generatr/build/install/structurizr-site-generatr/bin/structurizr-site-generatr

MAKEFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
TOP_PATH := $(dir $(MAKEFILE_PATH))

.PHONY: docker-run $(DOCKER_IMAGE) $(DROSIX_DL) $(DROSIX_OUTPUT) doc

docker-run: $(DOCKER_IMAGE) $(DROSIX_DL) $(DROSIX_OUTPUT) $(CARGO_REGISTRY)
	@$(DOCKER) run --rm -it --name $(DOCKER_NAME) -h $(DOCKER_NAME) \
		-v $(TOP_PATH):/home/worker/drosix \
		-v $(DROSIX_DL):/home/worker/dl \
		-v $(DROSIX_OUTPUT):/home/worker/output \
		-v $(CARGO_REGISTRY):/home/worker/.cargo/registry \
		-v $(TOP_PATH)/images:/home/worker/output/images \
		-p 8080:8080 \
		$(DOCKER_IMAGE)

$(DOCKER_IMAGE): $(DOCKER_BUILD_STAMP)
	@$(DOCKER) image inspect $@  > /dev/null || $(MAKE) $(DOCKER_BUILD_STAMP)

$(DOCKER_BUILD_STAMP): $(TOP_PATH)/docker/Dockerfile $(TOP_PATH)/docker/entrypoint.sh
	@$(DOCKER) build -t $(DOCKER_IMAGE) \
		--build-arg USER_ID=$(shell id -u) \
		--build-arg GROUP_ID=$(shell id -g) \
		$(TOP_PATH)/docker
	@touch $@

$(DROSIX_DL):
	@$(DOCKER) volume inspect $@ > /dev/null || $(DOCKER) volume create $@

$(DROSIX_OUTPUT):
	@$(DOCKER) volume inspect $@ > /dev/null || $(DOCKER) volume create $@

$(CARGO_REGISTRY):
	@$(DOCKER) volume inspect $@ > /dev/null || $(DOCKER) volume create $@

doc:
	@$(GENERATR) serve -w doc/workspace.dsl -a doc/assets -s doc/build -p 5555
