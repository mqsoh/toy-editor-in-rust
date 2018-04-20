DEV_IMAGE_NAME := dev/med
DEV_IMAGE_MARKER := .docker-dev-med

.PHONY: shell

$(DEV_IMAGE_MARKER): Dockerfile-dev
	docker build --tag $(DEV_IMAGE_NAME) -f $< .
	touch $(DEV_IMAGE_MARKER)

shell: $(DEV_IMAGE_MARKER)
	docker run --env USER --interactive --tty --rm --volume $$(pwd):/project-root --workdir /project-root $(DEV_IMAGE_NAME)
