import { Injectable, type ClassProvider } from '@nestjs/common';
import { Tool, type Context } from '@rekog/mcp-nest';
import type { Request } from 'express';
import { z } from 'zod';
import type { CallToolResult } from '@modelcontextprotocol/sdk/types.js';
import { execa } from 'execa';

import { ExecuteResultSchema, type Command, type ExecuteResult } from './commands.schema';

export function createCommandTool(name: string, spec: Command) {
  @Injectable()
  class DynamicCommandsTool {
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

      console.log(`$ ${command}`);

      const proc = execa(command, {
        shell: true,
        lines: true,
        all: true,
        reject: false,
      });

      const lines = [] as string[];

      for await (const line of proc.iterable({ from: 'all' })) {
        console.log(line);
        lines.push(line);
      }

      console.log(`> Exit code: ${proc.exitCode}`);

      const result: ExecuteResult = {
        command,
        code: proc.exitCode ?? 0,
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

export function createCommandToolProviders(commands: Record<string, Command>): ClassProvider[] {
  return Object.entries(commands).map(([name, spec]) => ({
    provide: getToolToken(name),
    useClass: createCommandTool(name, spec),
  }));
}

export function getToolToken(name: string): string {
  return `COMMAND(${name})`;
}
