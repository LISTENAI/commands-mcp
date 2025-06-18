import { Inject, Injectable } from '@nestjs/common';
import { Tool, type Context } from '@rekog/mcp-nest';
import type { Request } from 'express';
import { z } from 'zod';
import type { CallToolResult } from '@modelcontextprotocol/sdk/types.js';
import { execa } from 'execa';
import { $ as $kleur, bold, green, red, white } from 'kleur/colors';
import colorSupport from 'color-support';

import { WORKING_DIRECTORY_TOKEN } from './args.module';
import { ExecuteResultSchema, type Command, type ExecuteResult } from './commands.schema';

$kleur.enabled = !!colorSupport();

export function createCommandTool(name: string, spec: Command) {
  @Injectable()
  class DynamicCommandsTool {
    constructor(
      @Inject(WORKING_DIRECTORY_TOKEN) private readonly cwd: string,
    ) { }

    @Tool({
      name: name,
      description: spec.description,
      parameters: z.object(Object.fromEntries((spec.args ?? []).map((arg) => [
        arg.name,
        z.string().describe(arg.description),
      ]))),
      outputSchema: ExecuteResultSchema,
    })
    async execute(args: Record<string, string>, _context: Context, _req: Request): Promise<CallToolResult> {
      const command = spec.command.replace(/\{([^}]+)\}/g, (_, key) => {
        if (key in args) {
          return args[key]!;
        } else {
          throw new Error(`Missing argument: ${key}`);
        }
      });

      console.log(bold(white(`$ ${command}`)));

      const proc = execa(command, {
        shell: true,
        lines: true,
        all: true,
        reject: false,
        cwd: this.cwd,
      });

      const lines = [] as string[];

      for await (const line of proc.iterable({ from: 'all' })) {
        console.log(line);
        lines.push(line);
      }

      const code = proc.exitCode ?? 0;
      if (code == 0) {
        console.log(bold(green('> Command executed successfully')));
      } else {
        console.error(bold(red(`> Command failed with exit code ${code}`)));
      }

      const result: ExecuteResult = {
        command,
        code,
        output: lines.join('\n'),
      };

      return {
        content: [{
          type: 'text',
          text: JSON.stringify(result),
        }],
        structuredContent: result,
      };
    }
  }

  return DynamicCommandsTool;
}
