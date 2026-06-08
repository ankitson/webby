// webby configuration.
//
// Secrets / environment-specific values (Cloudflare account id, domains, the
// 1Password token reference) live in `.env.secret` (gitignored), NOT here.
// See `.env.secret.example` for the keys. Local paths and structural defaults
// stay in code since they're neither secret nor domain-specific.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

function loadEnvSecret() {
  const file = join(import.meta.dir, ".env.secret");
  if (!existsSync(file)) return;
  for (const raw of readFileSync(file, "utf8").split("\n")) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    const eq = line.indexOf("=");
    if (eq === -1) continue;
    const key = line.slice(0, eq).trim();
    let val = line.slice(eq + 1).trim();
    if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'"))) {
      val = val.slice(1, -1);
    }
    if (!(key in process.env)) process.env[key] = val;
  }
}
loadEnvSecret();

function req(key: string): string {
  const v = process.env[key];
  if (!v) {
    console.error(`✗ webby: missing ${key}. Set it in /projects/webby/.env.secret (see .env.secret.example).`);
    process.exit(1);
  }
  return v;
}

export interface Bag {
  /** short id used on the CLI (--bag <label>) */
  label: string;
  /** directory that holds the apps */
  dir: string;
  /** base URL the bag is served at */
  url: string;
  backend: "caddy" | "pages";
  /** Cloudflare Pages project name (pages backend only) */
  project?: string;
}

export const ACCOUNT_ID = req("CF_ACCOUNT_ID");

// 1Password secret reference for the Cloudflare API token (Pages: Edit).
// Read at deploy time via `op read`; the token itself never touches disk.
export const CF_TOKEN_REF = req("CF_TOKEN_REF");

export const BAGS: Record<string, Bag> = {
  internal: {
    label: "internal",
    dir: process.env.INTERNAL_DIR ?? join(import.meta.dir, "internal"),
    url: req("INTERNAL_URL"),
    backend: "caddy",
  },
  public: {
    label: "public",
    dir: process.env.PUBLIC_DIR ?? join(import.meta.dir, "public"),
    url: req("PUBLIC_URL"),
    backend: "pages",
    project: process.env.PUBLIC_PROJECT ?? "webby",
  },
};

export const DEFAULT_BAG = "internal";
