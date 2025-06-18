import { Module } from '@nestjs/common';
import { McpModule } from '@rekog/mcp-nest';

import { ArgumentsModule } from './args.module';
import { CommandsModule } from './commands.module';

import pkg from '../package.json';

@Module({
  imports: [
    McpModule.forRoot({
      name: pkg.name,
      version: pkg.version,
    }),
    ArgumentsModule.forRoot(),
    CommandsModule.forRootAsync('commands.yaml'),
  ],
})
export class AppModule {
}
