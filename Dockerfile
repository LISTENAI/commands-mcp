# syntax=docker/dockerfile:1

# stage: builder

FROM node:22-bookworm AS builder

WORKDIR /app

COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

# stage: runtime

FROM node:22-bookworm-slim

WORKDIR /app

COPY package*.json ./
RUN npm ci --omit=dev

COPY --from=builder /app/dist ./dist

ENTRYPOINT ["node", "dist/main.js"]
