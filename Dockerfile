# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the dependencies file
COPY Cargo.toml Cargo.lock ./

# Copy the rest of the application source code
COPY . .

# Build the application
RUN cargo build --release

# Create a new stage to create a smaller final image
FROM ubuntu:24.04

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the built executable from the builder stage
COPY --from=builder /usr/src/app/target/release/ferisss .

# Expose the port that your Rocket application will run on
EXPOSE 8000

# Command to run the application
CMD ["./ferisss"]
