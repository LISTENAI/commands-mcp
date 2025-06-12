declare module '*.hbs' {
  import type { HandlebarsTemplateDelegate } from 'handlebars';
  export default (() => string) as HandlebarsTemplateDelegate;
}
