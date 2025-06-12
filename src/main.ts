#!/usr/bin/env node
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';
import { parse } from 'yaml';
import { access, constants, readFile } from 'fs/promises';
import { join } from 'path';

import renderExploreCommands from '@/prompts/explore_commands.hbs';
import renderCommand from '@/prompts/command.hbs';

const COMMANDS_YAML = 'commands.yaml';

const server = new McpServer({
  name: 'MCP Server of collection of handful commands',
  version: '1.0.0',
});

interface CommandSpec {
  description: string;
  args?: {
    name: string;
    description: string;
    type: string;
    required?: boolean;
  }[];
  command: string;
}

async function readCommands(file: string): Promise<Record<string, CommandSpec>> {
  try {
    await access(file, constants.R_OK);
  } catch {
    return {};
  }

  return parse(await readFile(file, 'utf8'));
}

function buildCommandLine(command: CommandSpec, args: Record<string, string> = {}): string {
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
    const commands = await readCommands(join(cwd, COMMANDS_YAML));

    if (Object.keys(commands).length === 0) {
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
    const commands = await readCommands(join(cwd, COMMANDS_YAML));

    if (!commands[command]) {
      throw new Error(`Command not found: ${command}`);
    }

    return {
      content: [{
        type: 'text',
        text: renderCommand({ command: buildCommandLine(commands[command], args) }),
      }],
    };
  }
);

const transport = new StdioServerTransport();
await server.connect(transport);

console.error(`Server is running`);
