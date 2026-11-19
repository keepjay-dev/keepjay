// SPDX-License-Identifier: AGPL-3.0-or-later
// Compiles tokens.json to CSS custom properties: light by default, dark via
// prefers-color-scheme, and explicit [data-theme] overrides (07 §6).
// `--check` validates the token file without writing output.
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";

const tokens = JSON.parse(readFileSync(new URL("./tokens.json", import.meta.url), "utf8"));
const { light, dark } = tokens.color;

const missing = Object.keys(light).filter((k) => !(k in dark));
if (missing.length) {
  console.error(`dark theme is missing colour roles: ${missing.join(", ")}`);
  process.exit(1);
}
if (process.argv.includes("--check")) process.exit(0);

const vars = (obj, prefix) =>
  Object.entries(obj)
    .map(([k, v]) => `  --${prefix}-${k}: ${v};`)
    .join("\n");
const shared = [
  vars(tokens.space, "space"),
  vars(tokens.radius, "radius"),
  vars(tokens.font, "font"),
  vars(tokens.motion, "motion"),
].join("\n");

const css = `/* Generated from tokens.json by build.mjs. Do not edit. */
:root {
${vars(light, "color")}
${shared}
  color-scheme: light;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
${vars(dark, "color").replace(/^/gm, "  ")}
    color-scheme: dark;
  }
}
:root[data-theme="dark"] {
${vars(dark, "color")}
  color-scheme: dark;
}
@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-fast: 0ms;
    --motion-normal: 0ms;
  }
}
`;
mkdirSync(new URL("./dist/", import.meta.url), { recursive: true });
writeFileSync(new URL("./dist/tokens.css", import.meta.url), css);
console.log("design-tokens: wrote dist/tokens.css");
