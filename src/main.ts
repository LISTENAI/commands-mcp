#!/usr/bin/env node
import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import { z } from 'zod';
import { parse } from 'yaml';
import { readFileSync } from 'fs';

import { CollectionSchema } from '@/commands.schema';

const server = new McpServer({
  name: 'MCP Server of collection of handful commands',
  version: '1.0.0',
});

const [commandsFile] = process.argv.slice(2);

const { commands } = CollectionSchema.parse(parse(readFileSync(commandsFile!, 'utf8')));

for (const [name, command] of Object.entries(commands)) {
  server.tool(name, command.description, Object.fromEntries((command.args ?? []).map((arg) => [
    arg.name,
    z.string().describe(arg.description)
  ])), (args: Record<string, string>) => {
    const cmd = command.command.replace(/\{([^}]+)\}/g, (_, key) => {
      if (key in args) {
        return args[key]!;
      } else {
        throw new Error(`Missing argument: ${key}`);
      }
    });

    return {
      content: [{
        type: 'text',
        text: `Please execute the commands below:\n\n\`\`\`\n${cmd}\n\`\`\``,
      }],
    }
  });
}

const transport = new StdioServerTransport();
await server.connect(transport);

console.error(`Server is running`);
