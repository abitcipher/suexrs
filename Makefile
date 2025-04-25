.DEFAULT_GOAL := help

PROJECT_NAME = suexrs
DOCKER_USER = test
DOCKER_IMAGE = $(PROJECT_NAME)-image
DOCKER_CONTAINER = $(PROJECT_NAME)-container
RELEASE_TAG ?= v1.0.0

TARGET := ./target
TARGETWILD := $(call addprefix, $(TARGET), /*)

build: ##@ Build debug project
	@echo "Building debug project..."
	cargo build 

build-release: ##@ Build the release project
	@echo "Building release project..."
	cargo build --release:

check: ##@ Run source code check
	@echo "Run source code check..."
	cargo check 

clean: ##@ Clean the target directory
	rm -rf $(TARGETWILD)


docker-build: ##@ Build the Docker image
	@echo "Building Docker image $(DOCKER_IMAGE)..."
	$(eval RUSTVER=$(shell curl -s https://api.github.com/repos/rust-lang/rust/releases/latest | grep "tag_name" | grep -o "[0-9.]*"))
	docker build --no-cache --build-arg RUST_VERSION=$(RUSTVER) -t $(DOCKER_IMAGE) .


docker-run: ##@ Run the Docker container
	@echo "Running Docker image $(DOCKER_IMAGE)..."
	docker run -i -t --user $(DOCKER_USER) $(DOCKER_IMAGE) /bin/bash


.PHONY: help
help: ##@ Show this help
	@echo "Main commands:"
	@echo "  build: Build the project"
	@echo "  clean: Clean the target directory"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z0-9_.-]+:.*?##' $(MAKEFILE_LIST) | \
	sed 's/:.*##@/:/;s/##/:/' | \
	sort | \
	awk 'BEGIN {FS = ":"} {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'