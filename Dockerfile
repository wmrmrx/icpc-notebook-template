# Use the official Debian image as the base image
FROM debian:stable-slim

# Install required dependencies
RUN apt-get update && \
    apt-get install -y rustc wget xz-utils make && \
    wget https://github.com/typst/typst/releases/download/v0.10.0/typst-x86_64-unknown-linux-musl.tar.xz && \
    tar -xJf typst-x86_64-unknown-linux-musl.tar.xz && \
    mv typst-x86_64-unknown-linux-musl/typst /bin

# Set the working directory
WORKDIR /app

# Copy your application files into the container
COPY . .

VOLUME /app/output

# Your additional Dockerfile instructions go here

# Specify the default command to run when the container starts
CMD ["make", "pdf"]
