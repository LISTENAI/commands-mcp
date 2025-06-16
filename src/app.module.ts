import { Module } from '@nestjs/common';
import { McpModule } from '@rekog/mcp-nest';

import { CommandsModule } from './commands.module';

import pkg from '../package.json';

@Module({
  imports: [
    McpModule.forRoot({
      name: pkg.name,
      version: pkg.version,
    }),
    CommandsModule.forRootAsync('commands.yaml'),
  ],
})
export class AppModule {
}
