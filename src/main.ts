#!/usr/bin/env node
import { ConsoleLogger } from '@nestjs/common';
import { NestFactory } from '@nestjs/core';
import type { NestExpressApplication } from '@nestjs/platform-express';
import type { AddressInfo } from 'net';
import { access, constants } from 'fs/promises';

import { AppModule } from '@/app.module';
import { OPTIONS_TOKEN, resolvedManifestPath, WORKING_DIRECTORY_TOKEN, type Options } from '@/args.module';

(async () => {
  await access(resolvedManifestPath, constants.R_OK);

  const app = await NestFactory.create<NestExpressApplication>(AppModule, {
    logger: false,
  });

  const opts = app.get<Options>(OPTIONS_TOKEN);
  const cwd = app.get<string>(WORKING_DIRECTORY_TOKEN);

  if (opts.verbose) {
    app.useLogger(new ConsoleLogger());
  }

  if (opts.stdio) {
    const server = await app.listen(0);
    server.close();
    console.error('MCP Server is running in STDIO mode');
  } else {
    const server = await app.listen(opts.port ?? 0);
    console.error(`MCP Server is running at http://localhost:${(server.address() as AddressInfo).port}/sse`);
  }

  console.error(`Working directory: ${cwd}`);
})();
