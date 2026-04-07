export type ThemePreference = 'light' | 'dark' | 'system';
export type ResolvedTheme = 'light' | 'dark';

const STORAGE_KEY = 'diamond-theme';

function getSystemTheme(): ResolvedTheme {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return 'dark';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function loadPreference(): ThemePreference {
  if (typeof window === 'undefined') return 'system';
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'light' || stored === 'dark' || stored === 'system') return stored;
  return 'system';
}

let preference = $state<ThemePreference>(loadPreference());
let systemTheme = $state<ResolvedTheme>(getSystemTheme());

const resolved = $derived<ResolvedTheme>(
  preference === 'system' ? systemTheme : preference
);

export function getThemePreference(): ThemePreference {
  return preference;
}

export function getResolvedTheme(): ResolvedTheme {
  return resolved;
}

export function setThemePreference(next: ThemePreference): void {
  preference = next;
  localStorage.setItem(STORAGE_KEY, next);
  applyToDocument();
}

export function cycleTheme(): void {
  const order: ThemePreference[] = ['system', 'dark', 'light'];
  const idx = order.indexOf(preference);
  setThemePreference(order[(idx + 1) % order.length]);
}

function applyToDocument(): void {
  if (typeof document === 'undefined') return;
  const eff = preference === 'system' ? systemTheme : preference;
  document.documentElement.setAttribute('data-theme', eff);
  document.documentElement.style.colorScheme = eff;
}

// Listen for OS theme changes
if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
  const mq = window.matchMedia('(prefers-color-scheme: dark)');
  mq.addEventListener('change', (e) => {
    systemTheme = e.matches ? 'dark' : 'light';
    if (preference === 'system') applyToDocument();
  });

  // Apply on load
  applyToDocument();
}
