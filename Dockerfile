# Use the official Rust image as the base image
FROM rust:latest


# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the source code and Cargo.toml to the working directory
COPY . .
# Copy the environment file (.env) to the working directory
COPY .env .

# Build the Rust project
RUN cargo build --release

# Set the command to run your binary
RUN chmod 777 ./target/release/autoguard
CMD ["./target/release/autoguard"]
