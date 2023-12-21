# kill old
podman container stop autoguard
podman  container rm autoguard
podman  rmi autoguard
# decreased time for deployment
cargo clean
# create new
podman  build -t autoguard .
podman  run  \
  --name "autoguard" \
  --env-file .env \
autoguard .
echo "completed"



