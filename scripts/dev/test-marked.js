import { marked } from 'marked';
console.log(marked.parse('# Hello\n**Bold**', { async: false }));
