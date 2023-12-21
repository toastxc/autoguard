# Use the official Rust image as the base image
FROM registry.fedoraproject.org/fedora-minimal:latest



# Set the working directory inside the container
WORKDIR /app

# Copy the source code and Cargo.toml to the working directory
COPY ./target/release/autoguard .
# Copy the environment file (.env) to the working directory
COPY .env .

# Build the Rust project


ENTRYPOINT ["./autoguard"]
