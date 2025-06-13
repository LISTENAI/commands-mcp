import { zodToJsonSchema } from "zod-to-json-schema";

import { CollectionSchema } from '@/commands.schema';

const json = zodToJsonSchema(CollectionSchema, 'CollectionSchema');
console.log(JSON.stringify(json, null, 2));
