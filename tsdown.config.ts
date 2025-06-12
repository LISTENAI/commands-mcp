import { defineConfig } from 'tsdown'

import HandlebarsPlugin from './tsdown.plugin-hbs'

export default defineConfig({
  entry: './src/main.ts',
  plugins: [
    HandlebarsPlugin(),
  ],
})
