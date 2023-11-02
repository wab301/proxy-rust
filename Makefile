IMAGE = proxy-rust
IMAGE_REGISTRY = wab301

# This is the chart version. This version number should be incremented each time you make changes
# to the chart and its templates, including the app version.
# Versions are expected to follow Semantic Versioning (https://semver.org/)
VERSION = 1.0.0

TAG := $(VERSION)-$(shell git rev-parse --short=8 HEAD)

all: binary
.PHONY : all

# 仅编译二进制文件
binary:
	cargo build

# 构建镜像
build:
	docker build --build-arg ARG_HASH=$(TAG) -t $(IMAGE_REGISTRY)/$(IMAGE):$(TAG) ./

run:
	cargo run --bin proxy-rust

image: build clean
	docker push $(IMAGE_REGISTRY)/$(IMAGE):$(TAG)
	docker rmi $(IMAGE_REGISTRY)/$(IMAGE):$(TAG)