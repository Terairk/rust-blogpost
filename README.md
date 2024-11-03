
# JSON Expression Parser + Evaluator

## Overview

A Rust implementation of a simple one page blog post feed.

## Features

- Uses Axum + Postgres + SQLx
- Uses anyhow + thiserror crate for Error Handling
- Uses askama for templating HTML
- Tokio for async runtime

## Prerequisites

- Docker + Docker Compose (this can be done by installing Docker Desktop or otherwise)

## Building Commands
I used docker compose for handling building the main rust-web backend and the PostgreSQL database. The following commands are used for building and running the service.

For first-time build and after code changes:
```bash
docker compose up --build
```

For starting existing containers:
```bash
docker compose up
```

For stopping services:
```bash
docker compose down
```

For stopping services and removing volumes (database reset):
```bash
docker compose down --volumes
```

## Running the application
Open web browser and go to localhost:8080/home
You should then be able to fill in the form (use copy image address to get the link to a png from the web)
