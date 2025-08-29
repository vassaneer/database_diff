#!/bin/bash

# Script to build and run the db_diff application in Docker

# Check if docker is installed
if ! command -v docker &> /dev/null
then
    echo "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if docker-compose is installed
if ! command -v docker-compose &> /dev/null
then
    echo "docker-compose is not installed. Will use docker compose plugin."
fi

echo "Building the Docker image..."
if command -v docker-compose &> /dev/null
then
    docker-compose build
    echo "Starting the container..."
    docker-compose up -d
else
    docker compose build
    echo "Starting the container..."
    docker compose up -d
fi

echo "Application is now running on http://localhost:8080"
echo "You can stop it by running: docker-compose down (or docker compose down)"