# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim

ARG TARBALL
RUN --mount=type=bind,source=${TARBALL},target=/tmp/commands-mcp.tgz \
    npm install -g /tmp/commands-mcp.tgz

ENTRYPOINT [ "commands-mcp" ]
