##!/bin/bash
container="fedora-minimal-working-container"
buildah rmi localhost/autoguard

buildah from fedora-minimal:latest



mold --run cargo b -r

buildah run $container mkdir /server/

buildah copy $container .env /server/.env
buildah copy $container ./target/release/autoguard /server/autoguard

buildah run $container chmod 777 -R /server

buildah config --entrypoint "/server/autoguard -D FOREGROUND" $container
buildah commit $container autoguard

