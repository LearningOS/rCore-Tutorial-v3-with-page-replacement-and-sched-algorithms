DOCKER_NAME ?= rust-os-camp-2022
DIR := workplace
.PHONY: docker build_docker

flag = 
ifeq (${USER},1)
	flag += --rebuild-user
endif

run:
	cargo qemu --ch 8 ${flag} --sched ${SCHED}

docker:
	docker run --rm -it -v ${PWD}:/mnt -w /mnt ${DOCKER_NAME} bash

build_docker:
	docker build -t ${DOCKER_NAME} .

# for local ubuntu with zsh shell SHELL, need root for sudo
ubuntu_local_setenv:
	sudo apt-get update
	sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
              gawk build-essential bison flex texinfo gperf libtool patchutils bc \
              zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev git tmux python3 ninja-build zsh -y
	cd ${HOME} && wget https://download.qemu.org/qemu-7.0.0.tar.xz
	cd ${HOME} && tar xvJf qemu-7.0.0.tar.xz
	cd ${HOME}/qemu-7.0.0 && ./configure --target-list=riscv64-softmmu,riscv64-linux-user
	cd ${HOME}/qemu-7.0.0 && make -j$(nproc)
	cd ${HOME}/qemu-7.0.0 && sudo make install
	qemu-system-riscv64 --version
	qemu-riscv64 --version
	curl https://sh.rustup.rs -sSf | sh -s -- -y
	source ${HOME}/.cargo/env
	rustc --version

# for github codespaces ubuntu with zsh SHELL, need root for sudo
codespaces_setenv:
	sudo apt-get update
	sudo apt install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev \
              gawk build-essential bison flex texinfo gperf libtool patchutils bc \
              zlib1g-dev libexpat-dev pkg-config  libglib2.0-dev libpixman-1-dev git tmux python3 ninja-build zsh -y
	cd .. && wget https://download.qemu.org/qemu-7.0.0.tar.xz
	cd .. && tar xvJf qemu-7.0.0.tar.xz
	cd ../qemu-7.0.0 && ./configure --target-list=riscv64-softmmu,riscv64-linux-user
	cd ../qemu-7.0.0 && make -j$(nproc)
	cd ../qemu-7.0.0 && sudo make install
	qemu-system-riscv64 --version
	qemu-riscv64 --version
	curl https://sh.rustup.rs -sSf | sh -s -- -y
	/bin/zsh && source /home/codespace/.cargo/env
	rustc --version
