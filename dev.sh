podman-compose down
podman image rm autoguard_server
podman-compose up -d
podman logs autoguard_server_1