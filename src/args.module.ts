import { Module, type DynamicModule } from '@nestjs/common';
import { resolve } from 'path';
import { program } from 'commander';

import { description, version } from '../package.json';

program
  .name('commands-mcp')
  .description(description)
  .version(version)
  .option('-p, --port <number>', 'port to run the server on, 0 or omit for random port', parseInt)
  .option('-s, --stdio', 'run the server in STDIO mode', false)
  .option('-m, --manifest <path>', 'path to the commands manifest file', 'commands.yaml')
  .option('-v, --verbose', 'enable verbose logging', false)
  .argument('[working-directory]', 'the working directory to run commands in, defaults to $PWD')

// Claude Desktop 的 DXT 运行时会将 argv 伪造成 node 启动的样子，但由于
// process.versions.election 的存在，会被 commander.js 认出是 Electron 从而使用了错误
// 的 slice。所以强制指定为 node 模式。
// https://github.com/tj/commander.js/blob/395cf7145fe28122f5a69026b310e02df114f907/lib/command.js#L1010-L1012
program.parse(process.argv, { from: 'node' });

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
