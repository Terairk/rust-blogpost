
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

```bash
docker compose up --build (MAIN COMMAND - first time building and every code change)
docker compose up (for starting up again)
docker compose down (for stopping the service)
docker compose down --volumes (for removing volumes - ie mainly resetting the database)
```