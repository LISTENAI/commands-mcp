import type { DynamicModule } from '@nestjs/common';
import { readFile } from 'fs/promises';
import { parse } from 'yaml';

import { CollectionSchema } from './commands.schema';
import { createCommandToolProviders } from './commands.tool';

export class CommandsModule {
  static async forRootAsync(manifest: string): Promise<DynamicModule> {
    const { commands } = CollectionSchema.parse(parse(await readFile(manifest, 'utf8')));
    const providers = createCommandToolProviders(commands);
    return {
      module: CommandsModule,
      providers,
    };
  }
}
