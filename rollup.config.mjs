import typescript from '@rollup/plugin-typescript';

/** @type {import('rollup').RollupOptions} */
export default {
  input: 'src/main.ts',
  output: {
    file: 'dist/main.js',
    format: 'es',
  },
  plugins: [
    typescript(),
  ],
  external: [
    '@modelcontextprotocol/sdk/server/mcp.js',
    '@modelcontextprotocol/sdk/server/stdio.js',
    'zod',
  ],
};
