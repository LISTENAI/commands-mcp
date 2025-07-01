import { Module } from '@nestjs/common';
import { McpModule, McpTransportType } from '@rekog/mcp-nest';

import { ArgumentsModule, opts, resolvedManifestPath } from './args.module';
import { CommandsModule } from './commands.module';

import { name, version } from '../package.json';

@Module({
  imports: [
    McpModule.forRoot({
      name,
      version,
      transport: opts.stdio ? McpTransportType.STDIO : McpTransportType.SSE,
    }),
    ArgumentsModule.forRoot(),
    CommandsModule.forRootAsync(resolvedManifestPath),
  ],
})
export class AppModule {
}
