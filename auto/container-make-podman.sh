#!/bin/bash
# kill old
podman container stop autoguard
podman  container rm autoguard
podman  rmi autoguard
# decreased time for deployment
# create new
#podman  build -t autoguard .
sh auto/buildah.sh
podman  run -d \
  --name "autoguard" \
  --env-file .env \
autoguard
echo "completed"



