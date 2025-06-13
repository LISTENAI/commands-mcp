#!/usr/bin/env node
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';
import { parse } from 'yaml';
import { access, constants, readFile } from 'fs/promises';
import { join } from 'path';

import { CollectionSchema, CommandSchema } from '@/commands.schema';

import renderExploreCommands from '@/prompts/explore_commands.hbs';
import renderCommand from '@/prompts/command.hbs';

const COMMANDS_YAML = 'commands.yaml';

const server = new McpServer({
  name: 'MCP Server of collection of handful commands',
  version: '1.0.0',
});

type Collection = z.infer<typeof CollectionSchema>;
type Command = z.infer<typeof CommandSchema>;

async function readCommands(file: string): Promise<Collection> {
  try {
    await access(file, constants.R_OK);
  } catch {
    return { commands: [] };
  }

  return CollectionSchema.parse(parse(await readFile(file, 'utf8')));
}

function buildCommandLine(command: Command, args: Record<string, string> = {}): string {
  return command.command.replace(/\{([^\}]+)\}/g, (_, key) => {
    if (key in args) {
      return args[key]!;
    } else {
      throw new Error(`Missing argument: ${key}`);
    }
  });
}

server.tool(
  'explore_commands',
  'Explore available commands in the current working directory that may be useful for the current project',
  {
    cwd: z.string().describe('The current working directory'),
  },
  async ({ cwd }) => {
    const { commands } = await readCommands(join(cwd, COMMANDS_YAML));

    if (commands.length === 0) {
      return {
        content: [{
          type: 'text',
          text: 'No commands found in the current working directory.',
        }],
      };
    }

    return {
      content: [{
        type: 'text',
        text: renderExploreCommands({ commands }),
      }],
    };
  }
);

server.tool(
  'get_command',
  'Get a command to execute',
  {
    cwd: z.string().describe('The current working directory'),
    command: z.string().describe('The name of the command to get'),
    args: z.record(z.string()).optional().describe('Arguments for the command'),
  },
  async ({ cwd, command, args }) => {
    const { commands } = await readCommands(join(cwd, COMMANDS_YAML));
    const spec = commands.find(cmd => cmd.name === command);

    if (!spec) {
      throw new Error(`Command not found: ${command}`);
    }

    return {
      content: [{
        type: 'text',
        text: renderCommand({ command: buildCommandLine(spec, args) }),
      }],
    };
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);

console.error(`Server is running`);
