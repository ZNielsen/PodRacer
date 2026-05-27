IMAGE_NAME ?= podracer
IMAGE_TAG  ?= latest

.PHONY: docker docker-run docker-push

docker:
	docker build -t $(IMAGE_NAME):$(IMAGE_TAG) .

docker-run:
	docker run --rm -p 41968:41968 \
		-v podracer-data:/app/podcasts \
		$(IMAGE_NAME):$(IMAGE_TAG)

docker-push:
	docker push $(IMAGE_NAME):$(IMAGE_TAG)
