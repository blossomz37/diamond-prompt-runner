/**
 * Script: wcag-audit.mjs
 * Purpose: Audits predefined theme color pairs against WCAG 2.1 contrast thresholds.
 * Last modified: 2026-04-12
 */
// WCAG 2.1 contrast ratio calculator
function srgbToLinear(c) {
  return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

function luminance(hex) {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  return 0.2126 * srgbToLinear(r) + 0.7152 * srgbToLinear(g) + 0.0722 * srgbToLinear(b);
}

function contrast(hex1, hex2) {
  const l1 = luminance(hex1);
  const l2 = luminance(hex2);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

function grade(ratio, size) {
  if (size === "large") return ratio >= 4.5 ? "AAA" : ratio >= 3 ? "AA" : "FAIL";
  return ratio >= 7 ? "AAA" : ratio >= 4.5 ? "AA" : "FAIL";
}

// Flatten rgba onto a known opaque background
function blend(fgRgba, bgHex) {
  const m = fgRgba.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
  if (!m) return bgHex;
  const a = m[4] !== undefined ? parseFloat(m[4]) : 1;
  const br = parseInt(bgHex.slice(1, 3), 16);
  const bg = parseInt(bgHex.slice(3, 5), 16);
  const bb = parseInt(bgHex.slice(5, 7), 16);
  const r = Math.round(parseInt(m[1]) * a + br * (1 - a));
  const g = Math.round(parseInt(m[2]) * a + bg * (1 - a));
  const b = Math.round(parseInt(m[3]) * a + bb * (1 - a));
  return "#" + [r, g, b].map((c) => c.toString(16).padStart(2, "0")).join("");
}

function auditPairs(label, pairs) {
  console.log(`\n=== ${label} ===\n`);
  const failures = [];
  for (const [name, fg, bg, size] of pairs) {
    const r = contrast(fg, bg);
    const g = grade(r, size);
    const sizeNote = size === "large" ? " (large text)" : "";
    const icon = g === "FAIL" ? "  ✗" : g === "AA" ? "  ~" : "  ✓";
    console.log(`${icon} ${name.padEnd(32)} ${r.toFixed(2).padStart(5)}:1  ${g}${sizeNote}`);
    if (g === "FAIL") failures.push({ name, ratio: r.toFixed(2), fg, bg });
  }
  return failures;
}

// ── Dark theme ──
const darkBase = "#0b1120"; // gradient midpoint
const darkPanel = blend("rgba(18, 26, 44, 0.82)", darkBase);
const darkInput = blend("rgba(7, 11, 20, 0.82)", darkBase);
const darkCm = blend("rgba(5, 8, 15, 0.9)", darkBase);

console.log(`Dark surfaces: base=${darkBase}  panel=${darkPanel}  input=${darkInput}  cm=${darkCm}`);

const darkFails = auditPairs("DARK THEME", [
  ["--text on base",         "#f3f5ff", darkBase,  "normal"],
  ["--text on panel",        "#f3f5ff", darkPanel,  "normal"],
  ["--text on input",        "#f3f5ff", darkInput,  "normal"],
  ["--text-dim on base",     "#a4b0d0", darkBase,  "normal"],
  ["--text-dim on panel",    "#a4b0d0", darkPanel,  "normal"],
  ["--text-dim on input",    "#a4b0d0", darkInput,  "normal"],
  ["--text-soft on base",    "#8693b8", darkBase,  "normal"],
  ["--text-soft on panel",   "#8693b8", darkPanel,  "normal"],
  ["--text-soft on input",   "#8693b8", darkInput,  "normal"],
  ["--accent on base",       "#8bb1ff", darkBase,  "normal"],
  ["--accent on panel",      "#8bb1ff", darkPanel,  "normal"],
  ["--accent-strong on base","#b8cbff", darkBase,  "normal"],
  ["--danger on base",       "#ff8da1", darkBase,  "normal"],
  ["--danger on panel",      "#ff8da1", darkPanel,  "normal"],
  ["--success on base",      "#99e3be", darkBase,  "normal"],
  ["--success on panel",     "#99e3be", darkPanel,  "normal"],
  ["--cm-text on cm-bg",     "#dbe5ff", darkCm,    "normal"],
  // large text (>= 18px or 14px bold) — eyebrow labels, headings
  ["--text-soft on base (large)", "#8693b8", darkBase, "large"],
  ["--accent on base (large)",    "#8bb1ff", darkBase, "large"],
]);

// ── Light theme ──
const lightBase = "#eef0f5"; // gradient midpoint
const lightPanel = blend("rgba(255, 255, 255, 0.88)", lightBase);
const lightInput = blend("rgba(255, 255, 255, 0.9)", lightBase);

console.log(`\nLight surfaces: base=${lightBase}  panel=${lightPanel}  input=${lightInput}`);

const lightFails = auditPairs("LIGHT THEME", [
  ["--text on base",         "#1a1e2e", lightBase,  "normal"],
  ["--text on panel",        "#1a1e2e", lightPanel, "normal"],
  ["--text on input",        "#1a1e2e", lightInput, "normal"],
  ["--text-dim on base",     "#474d63", lightBase,  "normal"],
  ["--text-dim on panel",    "#474d63", lightPanel, "normal"],
  ["--text-dim on input",    "#474d63", lightInput, "normal"],
  ["--text-soft on base",    "#5c637a", lightBase,  "normal"],
  ["--text-soft on panel",   "#5c637a", lightPanel, "normal"],
  ["--text-soft on input",   "#5c637a", lightInput, "normal"],
  ["--accent on base",       "#2c5bb5", lightBase,  "normal"],
  ["--accent on panel",      "#2c5bb5", lightPanel, "normal"],
  ["--accent-strong on base","#1e4490", lightBase,  "normal"],
  ["--danger on base",       "#b5303f", lightBase,  "normal"],
  ["--danger on panel",      "#b5303f", lightPanel, "normal"],
  ["--success on base",      "#1e7048", lightBase,  "normal"],
  ["--success on panel",     "#1e7048", lightPanel, "normal"],
  ["--cm-text on cm-bg",     "#1a1e2e", "#fafbfe",  "normal"],
  // large text
  ["--text-soft on base (large)", "#5c637a", lightBase, "large"],
  ["--accent on base (large)",    "#2c5bb5", lightBase, "large"],
]);

// Summary
const allFails = [...darkFails, ...lightFails];
console.log("\n=== SUMMARY ===\n");
if (allFails.length === 0) {
  console.log("All pairs pass WCAG AA or better.");
} else {
  console.log(`${allFails.length} pair(s) FAIL WCAG AA:\n`);
  for (const f of allFails) {
    console.log(`  - ${f.name}: ${f.ratio}:1 (fg ${f.fg} on bg ${f.bg})`);
  }
}
