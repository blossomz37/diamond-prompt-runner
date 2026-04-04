import OpenAI from 'openai';

const apiKey = process.env.OPENROUTER_API_KEY;

if (!apiKey) {
  console.error('[probe:online] Missing OPENROUTER_API_KEY in the environment.');
  process.exit(1);
}

const model = process.env.OPENROUTER_ONLINE_MODEL ?? 'openai/gpt-5.4:online';
const maxResults = Number(process.env.OPENROUTER_WEB_MAX_RESULTS ?? '3');
const searchContextSize = process.env.OPENROUTER_SEARCH_CONTEXT_SIZE ?? 'medium';
const includeDomains = (process.env.OPENROUTER_INCLUDE_DOMAINS ?? '')
  .split(',')
  .map((value) => value.trim())
  .filter(Boolean);

const prompt =
  process.argv.slice(2).join(' ').trim() ||
  `Using web search, find one AI product or model announcement from the last 7 days. Today is ${new Date().toISOString().slice(0, 10)}. Respond in 3 short bullets and cite each source with markdown links.`;

const client = new OpenAI({
  apiKey,
  baseURL: 'https://openrouter.ai/api/v1',
  defaultHeaders: {
    'HTTP-Referer': 'https://github.com/blossomz37/diamond-prompt-runner',
    'X-OpenRouter-Title': 'Diamond Prompt Runner Online Probe'
  }
});

const plugin = {
  id: 'web',
  max_results: Number.isFinite(maxResults) ? maxResults : 3
};

if (includeDomains.length > 0) {
  plugin.include_domains = includeDomains;
}

try {
  const response = await client.chat.completions.create({
    model,
    messages: [
      {
        role: 'user',
        content: prompt
      }
    ],
    plugins: [plugin],
    web_search_options: {
      search_context_size: searchContextSize
    },
    temperature: 0.2
  });

  const message = response.choices[0]?.message;
  const output = Array.isArray(message?.content)
    ? message.content
        .map((part) => ('text' in part && typeof part.text === 'string' ? part.text : ''))
        .join('\n')
        .trim()
    : (message?.content ?? '').trim();

  const annotations = Array.isArray(message?.annotations) ? message.annotations : [];
  const webSearchRequests = response.usage?.server_tool_use?.web_search_requests ?? 0;

  if (!output) {
    console.error('[probe:online] OpenRouter returned an empty response body.');
    process.exit(1);
  }

  if (annotations.length === 0 && webSearchRequests === 0) {
    console.error('[probe:online] Request completed, but no web-search evidence was returned.');
    console.error(output);
    process.exit(1);
  }

  console.log(`[probe:online] Model: ${response.model}`);
  console.log(`[probe:online] Web search requests: ${webSearchRequests}`);
  console.log(`[probe:online] Citations: ${annotations.length}`);
  if (response.usage) {
    console.log(
      `[probe:online] Tokens: prompt=${response.usage.prompt_tokens} completion=${response.usage.completion_tokens} total=${response.usage.total_tokens}`
    );
    if (typeof response.usage.cost === 'number') {
      console.log(`[probe:online] Cost: ${response.usage.cost}`);
    }
  }
  console.log('');
  console.log(output);
} catch (error) {
  console.error(`[probe:online] ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}