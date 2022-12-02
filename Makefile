SIZE := 16
# DIR := .
DIR := /var/tmp/msm
NAME := $(DIR)/size$(SIZE)
# IMAGE := agfi-0d25a1d127f1b497f
IMAGE := agfi-09bec09a9e2b4d332
ifdef SIM
NO_DEFAULT_FEATURES := --no-default-features
endif
FLAGS := --release $(NO_DEFAULT_FEATURES)

CARGO := RUSTFLAGS='-C target-cpu=native' ~/.cargo/bin/cargo
SUDO_CARGO := RUSTFLAGS='-C target-cpu=native' sudo -E ~/.cargo/bin/cargo

default: msm

basic:
	$(SUDO_CARGO) run --release --example add
	$(SUDO_CARGO) run --release --example neg
	$(SUDO_CARGO) run --release --example sub

column: $(NAME).points $(NAME).beta
	$(SUDO_CARGO) run $(FLAGS) --example column -- $(SIZE) $(NAME)

column-pre: $(NAME).beta
	$(SUDO_CARGO) run $(FLAGS) --example column -- --preloaded $(SIZE) $(NAME)

load: $(NAME).beta $(NAME).points
	$(SUDO_CARGO) run $(FLAGS) --example load -- $(SIZE) $(NAME)

msm: $(NAME).points $(NAME).beta
	$(SUDO_CARGO) run $(FLAGS) --example msm -- $(SIZE) $(NAME)

msm-pre: $(NAME).beta
	$(SUDO_CARGO) run $(FLAGS) --example msm -- --preloaded $(SIZE) $(NAME)

points $(NAME).beta $(NAME).points:
	mkdir -p $(DIR)
	# $(CARGO) run --release --example points -- $(SIZE) $(NAME)
	$(SUDO_CARGO) run --release --example points -- $(SIZE) $(NAME)

reset:
	sudo fpga-load-local-image -S 0 -I $(IMAGE)

describe:
	aws ec2 describe-fpga-images --filters Name=fpga-image-global-id,Values=$(IMAGE)
