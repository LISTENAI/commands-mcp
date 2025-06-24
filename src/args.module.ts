import { Module, type DynamicModule } from '@nestjs/common';
import { resolve } from 'path';
import { program } from 'commander';

import pkg from '../package.json';

program
  .name('commands-mcp')
  .description(pkg.description)
  .version(pkg.version)
  .option('-p, --port <number>', 'port to run the server on, 0 or omit for random port', parseInt)
  .option('-s, --stdio', 'run the server in STDIO mode', false)
  .option('-m, --manifest <path>', 'path to the commands manifest file', 'commands.yaml')
  .option('-v, --verbose', 'enable verbose logging', false)
  .argument('[working-directory]', 'the working directory to run commands in, defaults to $PWD')

program.parse();

export const opts = program.opts<Options>();
export const args = program.args;

export interface Options {
  port?: number;
  stdio?: boolean;
  manifest: string;
  verbose?: boolean;
}

export const cwd = args[0] ?? process.cwd();
export const resolvedManifestPath = resolve(cwd, opts.manifest);

export const OPTIONS_TOKEN = Symbol('OPTIONS_TOKEN');
export const ARGUMENTS_TOKEN = Symbol('ARGUMENTS_TOKEN');
export const WORKING_DIRECTORY_TOKEN = Symbol('WORKING_DIRECTORY_TOKEN');

@Module({})
export class ArgumentsModule {
  static forRoot(): DynamicModule {
    return {
      module: ArgumentsModule,
      providers: [
        { provide: OPTIONS_TOKEN, useValue: opts },
        { provide: ARGUMENTS_TOKEN, useValue: args },
        { provide: WORKING_DIRECTORY_TOKEN, useValue: cwd },
      ],
      exports: [
        OPTIONS_TOKEN,
        ARGUMENTS_TOKEN,
        WORKING_DIRECTORY_TOKEN,
      ],
      global: true,
    };
  }
}
