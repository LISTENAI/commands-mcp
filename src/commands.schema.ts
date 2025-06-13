import { z } from 'zod';

export const CommandSchema = z.object({
  description: z.string().describe('A brief description of the command'),
  args: z.array(z.object({
    name: z.string().describe('The name of the argument'),
    description: z.string().describe('A brief description of the argument'),
    type: z.string().describe('The type of the argument (e.g., string, number, boolean)').default('string'),
    required: z.boolean().optional().describe('Whether the argument is required'),
  })).optional().describe('The arguments for the command'),
  command: z.string().describe('The command template with placeholders for arguments'),
});

export const CollectionSchema = z.object({
  commands: z.record(z.string(), CommandSchema).describe('A collection of commands'),
}).describe('A collection of commands with their specifications');
