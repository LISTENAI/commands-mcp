import { Module } from '@nestjs/common';
import { McpModule, McpTransportType } from '@rekog/mcp-nest';

import { ArgumentsModule, opts } from './args.module';
import { CommandsModule } from './commands.module';

import pkg from '../package.json';

@Module({
  imports: [
    McpModule.forRoot({
      name: pkg.name,
      version: pkg.version,
      transport: opts.stdio ? McpTransportType.STDIO : McpTransportType.SSE,
    }),
    ArgumentsModule.forRoot(),
    CommandsModule.forRootAsync(opts.manifest),
  ],
})
export class AppModule {
}
