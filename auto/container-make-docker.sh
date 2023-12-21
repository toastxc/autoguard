
# kill old
sudo docker container stop autoguard
sudo docker  container rm autoguard
sudo docker  rmi autoguard
# decreased time for deployment
mold --run cargo b -r
# create new
sudo docker  build -t autoguard .
sudo docker  run  -d \
  --name "autoguard" \
  --env-file .env \
autoguard .
echo "completed"



