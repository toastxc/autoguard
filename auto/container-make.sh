# kill old
podman container stop autoguard
podman container rm autoguard
podman rmi autoguard
# create new
cargo clean
podman build -t autoguard .
podman run  \
  --name "autoguard" \
  --env-file .env \
autoguard .



