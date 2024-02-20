# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the dependencies file
COPY Cargo.toml Cargo.lock ./

# Build the dependencies (this step allows Docker to cache the dependencies layer)
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && cargo build --release

# Remove the dummy source file
RUN rm -f src/main.rs

# Copy the rest of the application source code
COPY . .

# Build the application
RUN cargo build --release

# Create a new stage to create a smaller final image
FROM debian:buster-slim

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the built executable from the builder stage
COPY --from=builder /usr/src/app/target/release/ferisss .

# Expose the port that your Rocket application will run on
EXPOSE 8000

# Command to run the application
CMD ["./ferisss"]
