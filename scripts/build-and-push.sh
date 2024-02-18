#!/bin/sh
. ./.env.local
docker build -t "$DOCKER_IMAGE" . &&
docker push "$DOCKER_IMAGE"
