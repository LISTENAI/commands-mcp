#!/usr/bin/env node
import { ConsoleLogger } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import type { NestExpressApplication } from '@nestjs/platform-express';
import type { AddressInfo } from 'net';

import { AppModule } from '@/app.module';
import { OPTIONS_TOKEN, type Options } from '@/args.module';

const app = await NestFactory.create<NestExpressApplication>(AppModule, {
  logger: false,
});

const args = app.get<Options>(OPTIONS_TOKEN);

if (args.verbose) {
  app.useLogger(new ConsoleLogger());
}

const server = await app.listen(args.port ?? 0);
console.log(`MCP Server is running at http://localhost:${(server.address() as AddressInfo).port}/sse`);
