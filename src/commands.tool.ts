import { Inject, Injectable } from '@nestjs/common';
import { Tool, type Context } from '@rekog/mcp-nest';
import type { Request } from 'express';
import { z } from 'zod';
import type { CallToolResult } from '@modelcontextprotocol/sdk/types.js';
import Handlebars from 'handlebars';
import { execa } from 'execa';
import { $ as $kleur, bold, green, red, white } from 'kleur/colors';
import colorSupport from 'color-support';

import { WORKING_DIRECTORY_TOKEN } from './args.module';
import { ExecuteResultSchema, type Command, type ExecuteResult } from './commands.schema';

$kleur.enabled = !!colorSupport();

export function createCommandTool(name: string, spec: Command) {
  @Injectable()
  class DynamicCommandsTool {
    private readonly renderCommand = Handlebars.compile(spec.command);

    constructor(
      @Inject(WORKING_DIRECTORY_TOKEN) private readonly cwd: string,
    ) { }

    @Tool({
      name: name,
      description: spec.description,
      parameters: buildSchemaFromSpec(spec),
      outputSchema: ExecuteResultSchema,
    })
    async execute(args: Args, _context: Context, _req: Request): Promise<CallToolResult> {
      const command = this.renderCommand(args);

      console.log(bold(white(`$ ${command}`)));

      const proc = execa(command, {
        shell: true,
        lines: true,
        all: true,
        reject: false,
        cwd: this.cwd,
      });

      const terminateByTimeout = ((timeout) => timeout ? setTimeout(() => {
        console.error(bold(red(`> Command timed out after ${timeout}ms`)));
        proc.kill(spec.terminate?.signal);
      }, timeout) : undefined)(spec.terminate?.timeout);

      const terminateByOutput = ((pattern) => {
        if (!pattern) {
          return undefined;
        } else if (pattern.startsWith('/') && pattern.endsWith('/')) {
          const regex = new RegExp(pattern.slice(1, -1));
          return (line: string) => {
            if (regex.test(line)) {
              console.error(bold(red(`> Command terminated due to output matching: ${pattern}`)));
              proc.kill(spec.terminate?.signal);
            }
          };
        } else {
          return (line: string) => {
            if (line.includes(pattern)) {
              console.error(bold(red(`> Command terminated due to output containing: ${pattern}`)));
              proc.kill(spec.terminate?.signal);
            }
          };
        }
      })(spec.terminate?.output);

      const lines = [] as string[];

      for await (const line of proc.iterable({ from: 'all' })) {
        console.log(line);
        lines.push(line);
        terminateByOutput?.(line);
      }

      clearTimeout(terminateByTimeout);

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

function buildSchemaFromSpec(spec: Command) {
  return z.object(Object.fromEntries((spec.args ?? []).map((arg) => {
    let schema: z.ZodType = z[arg.type]().describe(arg.description);

    if (!arg.required) {
      schema = schema.optional();
    }

    if (arg.default !== undefined) {
      schema = schema.default(arg.default);
    }

    return [arg.name, schema];
  })));
}

type Args = z.infer<ReturnType<typeof buildSchemaFromSpec>>;
