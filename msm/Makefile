SIZE := 16
DIR := /var/tmp/cyclone
NAME := $(DIR)/size$(SIZE)
# IMAGE := agfi-0d25a1d127f1b497f
IMAGE := agfi-09bec09a9e2b4d332
# IMAGE := agfi-0247e0259194e6ce0

# deactivate the `hw` feature if SIM is set
ifdef SIM
NO_DEFAULT_FEATURES := --no-default-features
endif
FLAGS := --release $(NO_DEFAULT_FEATURES) --features demo

CARGO := RUSTFLAGS='-C target-cpu=native' cargo
# SUDO_CARGO := RUSTFLAGS='-C target-cpu=native' sudo $(shell command -v cargo)

MSM = target/release/cyclone-msm

default: msm

basic:
	$(CARGO) build --release --example identity
	sudo target/release/examples/identity
	$(CARGO) build --release --example add
	sudo target/release/examples/add
	$(CARGO) build --release --example neg
	sudo target/release/examples/neg
	$(CARGO) build --release --example sub
	sudo target/release/examples/sub

cyclone-msm:
	$(CARGO) build $(FLAGS) --bin cyclone-msm

install:
	$(CARGO) install --path . --features demo

git-install:
	CARGO_NET_GIT_FETCH_WITH_CLI=true $(CARGO) install --features demo --git 'ssh://git@github.com/nickray/cyclone-msm.git'


column: cyclone-msm
	sudo $(MSM) $(SIZE) $(NAME) column

column-pre: cyclone-msm
	sudo $(MSM) --preloaded $(SIZE) $(NAME) column

load: cyclone-msm
	sudo $(MSM) $(SIZE) $(NAME) load

msm: cyclone-msm
	sudo $(MSM) $(SIZE) $(NAME) msm

msm-pre: cyclone-msm
	sudo $(MSM) --preloaded $(SIZE) $(NAME) msm

points: cyclone-msm
	mkdir -p $(DIR)
	$(MSM) $(SIZE) $(NAME) points


reset:
	sudo fpga-load-local-image -S 0 -I $(IMAGE)

describe:
	aws ec2 describe-fpga-images --filters Name=fpga-image-global-id,Values=$(IMAGE)
