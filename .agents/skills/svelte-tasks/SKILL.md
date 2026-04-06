---
name: svelte
description: Use this skill for Svelte and SvelteKit tasks, including component work, utilities, debugging, migrations, project setup, and framework-specific implementation guidance.
---

# Svelte Skill

## Purpose

This skill provides operating guidance for Svelte and SvelteKit work. It defines how to approach Svelte-related tasks, which MCP tools to use, when to use them, and how to validate code before returning it.

Use this skill whenever the task involves:
- Svelte components
- Svelte modules
- SvelteKit routes, load functions, forms, hooks, adapters, or configuration
- Svelte migrations
- debugging Svelte or SvelteKit issues
- framework-specific implementation decisions
- generating example code for Svelte developers

## Operating rules

1. Start with direct reasoning.
   - First answer from your own Svelte knowledge when the solution is straightforward.
   - Use MCP documentation selectively when the task depends on framework-specific details, newer APIs, migration edge cases, or exact recommended patterns.

2. Prefer efficient documentation access.
   - The documentation index below is already available.
   - Do not call a section-listing tool again if the same information is already present.
   - Retrieve only the sections that are relevant to the current task, since documentation fetches are token-intensive.

3. Validate every Svelte code artifact with the autofixer.
   - Every time you write or revise a Svelte component or Svelte module, run `svelte-autofixer` on that code.
   - If the tool returns issues or suggestions, fix them and run it again.
   - Continue until the tool returns no issues or suggestions.
   - Only then return the final code.

4. Keep the implementation aligned with the task.
   - Favor idiomatic Svelte and SvelteKit patterns.
   - Match the project’s apparent version and conventions when they are visible.
   - Use migration-aware reasoning when the codebase appears to mix legacy and runes-era syntax.

5. Offer a playground link only at the end.
   - If code is being shared in chat rather than written into files, offer to generate a playground link after the code is finalized.
   - Generate the playground link only once the final code is ready.
   - The playground payload must include an entry file named `App.svelte`.
   - If multiple files are needed, include them all at the root.

## Recommended workflow

For a Svelte-related task, follow this sequence:

1. Determine whether the task is:
   - component implementation
   - module or utility implementation
   - debugging
   - migration
   - setup or configuration
   - testing
   - deployment or adapter work

2. Solve from existing knowledge first.

3. If framework documentation is needed:
   - call `get-documentation` with the smallest relevant set of paths
   - avoid broad retrieval
   - prioritize exact sections that match the task

4. Write or revise the code.

5. Run `svelte-autofixer` on the resulting Svelte code.

6. Apply fixes and rerun until clean.

7. Return the final answer.

8. If the code is being delivered in chat, ask whether the user wants a playground link.

## Tools used by this skill

### `get-documentation`
Use this tool when the task requires authoritative Svelte or SvelteKit documentation, especially for:
- exact API behavior
- current recommended patterns
- migration details
- framework conventions
- adapters, hooks, or configuration
- compiler/runtime warnings and errors

### `svelte-autofixer`
Use this tool every time you produce or revise:
- a `.svelte` component
- a Svelte module
- code intended for direct use in a Svelte or SvelteKit project

This tool is mandatory for final validation.

### `playground-link`
Use this only after the final code is complete and validated, and only if the user wants a runnable playground link.

## Documentation index

The following documentation paths are available for targeted retrieval. Use title and path to estimate relevance.

<available-docs>

- title: Overview, use_cases: use title and path to estimate use case, path: ai/overview
- title: Local setup, use_cases: use title and path to estimate use case, path: ai/local-setup
- title: Remote setup, use_cases: use title and path to estimate use case, path: ai/remote-setup
- title: Tools, use_cases: use title and path to estimate use case, path: ai/tools
- title: Resources, use_cases: use title and path to estimate use case, path: ai/resources
- title: Prompts, use_cases: use title and path to estimate use case, path: ai/prompts
- title: Overview, use_cases: use title and path to estimate use case, path: ai/plugin
- title: Subagent, use_cases: use title and path to estimate use case, path: ai/subagent
- title: Overview, use_cases: use title and path to estimate use case, path: ai/opencode-plugin
- title: Subagent, use_cases: use title and path to estimate use case, path: ai/opencode-subagent
- title: Overview, use_cases: use title and path to estimate use case, path: ai/skills
- title: Overview, use_cases: project setup, creating new svelte apps, scaffolding, cli tools, initializing projects, path: cli/overview
- title: Frequently asked questions, use_cases: project setup, initializing new svelte projects, troubleshooting cli installation, package manager configuration, path: cli/faq
- title: sv create, use_cases: project setup, starting new sveltekit app, initializing project, creating from playground, choosing project template, path: cli/sv-create
- title: sv add, use_cases: project setup, adding features to existing projects, integrating tools, testing setup, styling setup, authentication, database setup, deployment adapters, path: cli/sv-add
- title: sv check, use_cases: code quality, ci/cd pipelines, error checking, typescript projects, pre-commit hooks, finding unused css, accessibility auditing, production builds, path: cli/sv-check
- title: sv migrate, use_cases: migration, upgrading svelte versions, upgrading sveltekit versions, modernizing codebase, svelte 3 to 4, svelte 4 to 5, sveltekit 1 to 2, adopting runes, refactoring deprecated apis, path: cli/sv-migrate
- title: devtools-json, use_cases: development setup, chrome devtools integration, browser-based editing, local development workflow, debugging setup, path: cli/devtools-json
- title: drizzle, use_cases: database setup, sql queries, orm integration, data modeling, postgresql, mysql, sqlite, server-side data access, database migrations, type-safe queries, path: cli/drizzle
- title: eslint, use_cases: code quality, linting, error detection, project setup, code standards, team collaboration, typescript projects, path: cli/eslint
- title: better-auth, use_cases: use title and path to estimate use case, path: cli/better-auth
- title: mcp, use_cases: use title and path to estimate use case, path: cli/mcp
- title: mdsvex, use_cases: blog, content sites, markdown rendering, documentation sites, technical writing, cms integration, article pages, path: cli/mdsvex
- title: paraglide, use_cases: internationalization, multi-language sites, i18n, translation, localization, language switching, global apps, multilingual content, path: cli/paraglide
- title: playwright, use_cases: browser testing, e2e testing, integration testing, test automation, quality assurance, ci/cd pipelines, testing user flows, path: cli/playwright
- title: prettier, use_cases: code formatting, project setup, code style consistency, team collaboration, linting configuration, path: cli/prettier
- title: storybook, use_cases: component development, design systems, ui library, isolated component testing, documentation, visual testing, component showcase, path: cli/storybook
- title: sveltekit-adapter, use_cases: deployment, production builds, hosting setup, choosing deployment platform, configuring adapters, static site generation, node server, vercel, cloudflare, netlify, path: cli/sveltekit-adapter
- title: tailwindcss, use_cases: project setup, styling, css framework, rapid prototyping, utility-first css, design systems, responsive design, adding tailwind to svelte, path: cli/tailwind
- title: vitest, use_cases: testing, unit tests, component testing, test setup, quality assurance, ci/cd pipelines, test-driven development, path: cli/vitest

- title: Introduction, use_cases: learning sveltekit, project setup, understanding framework basics, choosing between svelte and sveltekit, getting started with full-stack apps, path: kit/introduction
- title: Creating a project, use_cases: project setup, starting new sveltekit app, initial development environment, first-time sveltekit users, scaffolding projects, path: kit/creating-a-project
- title: Project types, use_cases: deployment, project setup, choosing adapters, ssg, spa, ssr, serverless, mobile apps, desktop apps, pwa, offline apps, browser extensions, separate backend, docker containers, path: kit/project-types
- title: Project structure, use_cases: project setup, understanding file structure, organizing code, starting new project, learning sveltekit basics, path: kit/project-structure
- title: Web standards, use_cases: always, any sveltekit project, data fetching, forms, api routes, server-side rendering, deployment to various platforms, path: kit/web-standards
- title: Routing, use_cases: routing, navigation, multi-page apps, project setup, file structure, api endpoints, data loading, layouts, error pages, always, path: kit/routing
- title: Loading data, use_cases: data fetching, api calls, database queries, dynamic routes, page initialization, loading states, authentication checks, ssr data, form data, content rendering, path: kit/load
- title: Form actions, use_cases: forms, user input, data submission, authentication, login systems, user registration, progressive enhancement, validation errors, path: kit/form-actions
- title: Page options, use_cases: prerendering static sites, ssr configuration, spa setup, client-side rendering control, url trailing slash handling, adapter deployment config, build optimization, path: kit/page-options
- title: State management, use_cases: sveltekit, server-side rendering, ssr, state management, authentication, data persistence, load functions, context api, navigation, component lifecycle, path: kit/state-management
- title: Remote functions, use_cases: data fetching, server-side logic, database queries, type-safe client-server communication, forms, user input, mutations, authentication, crud operations, optimistic updates, path: kit/remote-functions
- title: Building your app, use_cases: production builds, deployment preparation, build process optimization, adapter configuration, preview before deployment, path: kit/building-your-app
- title: Adapters, use_cases: deployment, production builds, hosting setup, choosing deployment platform, configuring adapters, path: kit/adapters
- title: Hooks, use_cases: authentication, logging, error tracking, request interception, api proxying, custom routing, internationalization, database initialization, middleware logic, session management, path: kit/hooks
- title: Errors, use_cases: error handling, custom error pages, 404 pages, api error responses, production error logging, error tracking, type-safe errors, path: kit/errors
- title: Server-only modules, use_cases: api keys, environment variables, sensitive data protection, backend security, preventing data leaks, server-side code isolation, path: kit/server-only-modules
- title: Observability, use_cases: performance monitoring, debugging, observability, tracing requests, production diagnostics, analyzing slow requests, finding bottlenecks, monitoring server-side operations, path: kit/observability
- title: Performance, use_cases: performance optimization, slow loading pages, production deployment, debugging performance issues, reducing bundle size, improving load times, path: kit/performance
- title: Accessibility, use_cases: always, any sveltekit project, screen reader support, keyboard navigation, multi-page apps, client-side routing, internationalization, multilingual sites, path: kit/accessibility
- title: SEO, use_cases: seo optimization, search engine ranking, content sites, blogs, marketing sites, public-facing apps, sitemaps, amp pages, meta tags, performance optimization, path: kit/seo
- title: Integrations, use_cases: project setup, css preprocessors, postcss, scss, sass, less, stylus, typescript setup, adding integrations, tailwind, testing, auth, linting, formatting, path: kit/integrations
- title: Migrating to SvelteKit v2, use_cases: migration, upgrading from sveltekit 1 to 2, breaking changes, version updates, path: kit/migrating-to-sveltekit-2

- title: Overview, use_cases: always, any svelte project, getting started, learning svelte, introduction, project setup, understanding framework basics, path: svelte/overview
- title: Getting started, use_cases: project setup, starting new svelte project, initial installation, choosing between sveltekit and vite, editor configuration, path: svelte/getting-started
- title: .svelte files, use_cases: always, any svelte project, component creation, project setup, learning svelte basics, path: svelte/svelte-files
- title: .svelte.js and .svelte.ts files, use_cases: shared reactive state, reusable reactive logic, state management across components, global stores, custom reactive utilities, path: svelte/svelte-js-files
- title: What are runes?, use_cases: always, any svelte 5 project, understanding core syntax, learning svelte 5, migration from svelte 4, path: svelte/what-are-runes
- title: $state, use_cases: always, any svelte project, core reactivity, state management, counters, forms, todo apps, interactive ui, data updates, class-based components, path: svelte/$state
- title: $derived, use_cases: always, any svelte project, computed values, reactive calculations, derived data, transforming state, dependent values, path: svelte/$derived
- title: $effect, use_cases: canvas drawing, third-party library integration, dom manipulation, side effects, intervals, timers, network requests, analytics tracking, path: svelte/$effect
- title: $props, use_cases: always, any svelte project, passing data to components, component communication, reusable components, component props, path: svelte/$props
- title: Basic markup, use_cases: always, any svelte project, basic markup, html templating, component structure, attributes, events, props, text rendering, path: svelte/basic-markup
- title: {#if ...}, use_cases: always, conditional rendering, showing/hiding content, dynamic ui, user permissions, loading states, error handling, form validation, path: svelte/if
- title: {#each ...}, use_cases: always, lists, arrays, iteration, product listings, todos, tables, grids, dynamic content, shopping carts, user lists, comments, feeds, path: svelte/each
- title: {#await ...}, use_cases: async data fetching, api calls, loading states, promises, error handling, lazy loading components, dynamic imports, path: svelte/await
- title: {@render ...}, use_cases: reusable ui patterns, component composition, conditional rendering, fallback content, layout components, slot alternatives, template reuse, path: svelte/@render
- title: bind:, use_cases: forms, user input, two-way data binding, interactive ui, media players, file uploads, checkboxes, radio buttons, select dropdowns, contenteditable, dimension tracking, path: svelte/bind
- title: transition:, use_cases: animations, interactive ui, modals, dropdowns, notifications, conditional content, show/hide elements, smooth state changes, path: svelte/transition
- title: animate:, use_cases: sortable lists, drag and drop, reorderable items, todo lists, kanban boards, playlist editors, priority queues, animated list reordering, path: svelte/animate
- title: Scoped styles, use_cases: always, styling components, scoped css, component-specific styles, preventing style conflicts, animations, keyframes, path: svelte/scoped-styles
- title: Global styles, use_cases: global styles, third-party libraries, css resets, animations, styling body/html, overriding component styles, shared keyframes, base styles, path: svelte/global-styles
- title: <svelte:head>, use_cases: seo optimization, page titles, meta tags, social media sharing, dynamic head content, multi-page apps, blog posts, product pages, path: svelte/svelte-head
- title: Stores, use_cases: shared state, cross-component data, reactive values, async data streams, manual control over updates, rxjs integration, extracting logic, path: svelte/stores
- title: Context, use_cases: shared state, avoiding prop drilling, component communication, theme providers, user context, authentication state, configuration sharing, deeply nested components, path: svelte/context
- title: Lifecycle hooks, use_cases: component initialization, cleanup tasks, timers, subscriptions, dom measurements, chat windows, autoscroll features, migration from svelte 4, path: svelte/lifecycle-hooks
- title: Best practices, use_cases: use title and path to estimate use case, path: svelte/best-practices
- title: Testing, use_cases: testing, quality assurance, unit tests, integration tests, component tests, e2e tests, vitest setup, playwright setup, test automation, path: svelte/testing
- title: TypeScript, use_cases: typescript setup, type safety, component props typing, generic components, wrapper components, dom type augmentation, project configuration, path: svelte/typescript
- title: Svelte 5 migration guide, use_cases: migrating from svelte 4 to 5, upgrading projects, learning svelte 5 syntax changes, runes migration, event handler updates, path: svelte/v5-migration-guide
- title: Compiler errors, use_cases: debugging compile failures, invalid syntax, unsupported constructs, path: svelte/compiler-errors
- title: Compiler warnings, use_cases: accessibility, semantic correctness, quality review, path: svelte/compiler-warnings
- title: Runtime errors, use_cases: debugging errors, error handling, troubleshooting runtime issues, path: svelte/runtime-errors
- title: Runtime warnings, use_cases: debugging state proxies, console logging reactive values, inspecting state changes, development troubleshooting, path: svelte/runtime-warnings

</available-docs>

## Execution template

When this skill is invoked, apply the following task frame internally:

<task>
[DESCRIBE THE CURRENT SVELTE TASK HERE]
</task>