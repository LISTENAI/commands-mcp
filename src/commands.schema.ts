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

export type Command = z.infer<typeof CommandSchema>;

export const CollectionSchema = z.object({
  commands: z.record(z.string(), CommandSchema).describe('A collection of commands'),
}).describe('A collection of commands with their specifications');

export type Collection = z.infer<typeof CollectionSchema>;

export const ExecuteResultSchema = z.object({
  command: z.string().describe('The command that was executed'),
  code: z.number().describe('The exit code of the command'),
  output: z.string().describe('The output of the command'),
});

export type ExecuteResult = z.infer<typeof ExecuteResultSchema>;
