
# kill old
sudo docker container stop autoguard
sudo docker  container rm autoguard
sudo docker  rmi autoguard
# decreased time for deployment
cargo clean
# create new
sudo docker  build -t autoguard .
sudo docker  run  \
  --name "autoguard" \
  --env-file .env \
autoguard .
echo "completed"



