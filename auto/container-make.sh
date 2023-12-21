# I personally prefer podman but this script will work for either platform
echo "set the container engine to Docker if that's what you use, podman is default"
engine="podman"
# kill old
$engine container stop autoguard
$engine container rm autoguard
$engine rmi autoguard
# decreased time for deployment
cargo clean
# create new
$engine build -t autoguard .
$engine run  \
  --name "autoguard" \
  --env-file .env \
autoguard .
echo "completed"



