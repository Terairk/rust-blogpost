
# JSON Expression Parser + Evaluator

## Overview

A Rust implementation of a simple one page blog post feed.

## Features

- Uses Axum + Postgres + SQLx
- Uses anyhow + thiserror crate for Error Handling
- Uses askama for templating HTML
- Tokio for async runtime

## Prerequisites

- Docker + Docker Compose
- Potentially PostgreSQL (but should be handled by Docker)

## Building Commands
I used docker compose for handling building the main rust-web backend and the PostgreSQL database. The following commands are used for building and running the service.

```bash
docker compose up --build (MAIN COMMAND - first time building and every code change)
docker compose up (for starting up again)
docker compose down (for stopping the service)
docker compose down --volumes (for removing volumes - ie mainly resetting the database)
```

## Running the application
Open web browser and go to localhost:8080/home
You should then be able to fill in the form (use copy image address to get the link to a png from the web)
