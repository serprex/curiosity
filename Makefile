USER_ROOT := $(abspath .)

export DOCKER_HOST=192.168.99.100:2376 # docker-machine default
export DOCKER_CERT_PATH=USER_ROOT/.docker/machine/certs # docker-machine certs

default: build

build:
	cargo build
