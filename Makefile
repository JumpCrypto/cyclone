SIZE := 16
# DIR := .
DIR := /var/tmp/msm
NAME := $(DIR)/size$(SIZE)
# IMAGE := agfi-0d25a1d127f1b497f
IMAGE := agfi-09bec09a9e2b4d332

# deactivate the `hw` feature if SIM is set
ifdef SIM
NO_DEFAULT_FEATURES := --no-default-features
endif
FLAGS := --release $(NO_DEFAULT_FEATURES) --features demo

CARGO := RUSTFLAGS='-C target-cpu=native' cargo
# it seems `sudo -E` does *not* preserve PATH, which is what we want
SUDO_CARGO := RUSTFLAGS='-C target-cpu=native' sudo --preserve-env=PATH cargo

COLUMN = target/release/cyclone-msm-column
LOAD = target/release/cyclone-msm-load
MSM = target/release/cyclone-msm
POINTS = target/release/cyclone-msm-points

default: msm

basic:
	$(SUDO_CARGO) run --release --example add
	$(SUDO_CARGO) run --release --example neg
	$(SUDO_CARGO) run --release --example sub

cyclone-msm:
	$(CARGO) build $(FLAGS) --bin cyclone-msm

cyclone-msm-column:
	$(CARGO) build $(FLAGS) --bin cyclone-msm-column

cyclone-msm-load:
	$(CARGO) build $(FLAGS) --bin cyclone-msm-load

cyclone-msm-points:
	$(CARGO) build $(FLAGS) --bin cyclone-msm-points

install:
	$(CARGO) install --path . --features demo

column: cyclone-msm-column
	sudo $(COLUMN) $(SIZE) $(NAME)

column-pre: cyclone-msm-column
	sudo $(COLUMN) --preloaded $(SIZE) $(NAME)

load: cyclone-msm-load
	sudo $(LOAD) $(SIZE) $(NAME)

msm: cyclone-msm
	sudo $(MSM) $(SIZE) $(NAME)

msm-pre: cyclone-msm
	sudo $(MSM) --preloaded $(SIZE) $(NAME)

points: cyclone-msm-points
	mkdir -p $(DIR)
	$(POINTS) $(SIZE) $(NAME)


reset:
	sudo fpga-load-local-image -S 0 -I $(IMAGE)

describe:
	aws ec2 describe-fpga-images --filters Name=fpga-image-global-id,Values=$(IMAGE)
