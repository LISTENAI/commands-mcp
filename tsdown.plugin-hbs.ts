import type { RolldownPlugin } from 'rolldown';
import { precompile } from 'handlebars';

export default function HandlebarsPlugin(): RolldownPlugin {
  return {
    name: 'handlebars',
    transform: {
      filter: {
        id: /\.(hbs|handlebars)$/,
      },
      handler(code) {
        return `import Handlebars from 'handlebars'; export default Handlebars.template(${precompile(code)});`;
      },
    },
  };
}
