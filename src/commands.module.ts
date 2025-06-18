import type { ClassProvider, DynamicModule } from '@nestjs/common';
import { readFile } from 'fs/promises';
import { parse } from 'yaml';

import { CollectionSchema, type Command } from './commands.schema';
import { createCommandTool } from './commands.tool';

export class CommandsModule {
  static async forRootAsync(manifest: string): Promise<DynamicModule> {
    const { commands } = CollectionSchema.parse(parse(await readFile(manifest, 'utf8')));

    const providers = Object.entries(commands).map(([name, spec]) =>
      this.createCommandToolProvider(name, spec));

    return {
      module: CommandsModule,
      providers,
    };
  }

  static createCommandToolProvider(name: string, spec: Command): ClassProvider {
    return {
      provide: `COMMAND(${name})`,
      useClass: createCommandTool(name, spec),
    };
  }
}
