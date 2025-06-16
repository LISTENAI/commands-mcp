#!/usr/bin/env node
import { ConsoleLogger } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import type { NestExpressApplication } from '@nestjs/platform-express';
import type { AddressInfo } from 'net';

import { AppModule } from '@/app.module';

const PORT = process.env.PORT ? parseInt(process.env.PORT) : 0;
const DEBUG = process.env.DEBUG == '1';

const app = await NestFactory.create<NestExpressApplication>(AppModule, {
  logger: DEBUG ? new ConsoleLogger() : false,
});

const server = await app.listen(PORT);
console.log(`MCP Server is running at http://localhost:${(server.address() as AddressInfo).port}/sse`);
