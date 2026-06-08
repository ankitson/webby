// webby configuration.
//
// The OSS default is intentionally no-setup: a local bag served from
// ~/.local/share/webby/local. Environment variables and an optional JSON config
// can add tailnet, Cloudflare Pages, Caddy, or custom command-backed bags.

import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { homedir } from "node:os";

export type Host =
  | { type: "local"; url?: string; port?: number }
  | { type: "caddy"; url?: string }
  | { type: "tailscale-serve"; url?: string; path?: string; background?: boolean }
  | { type: "tailscale-funnel"; url?: string; path?: string; background?: boolean }
  | {
      type: "cloudflare-pages";
      url?: string;
      project?: string;
      accountId?: string;
      tokenEnv?: string;
      tokenRef?: string;
      tokenCommand?: string;
    }
  | { type: "command"; url?: string; deploy?: string; afterAdd?: string; open?: string };

export interface Bag {
  /** short id used on the CLI (--bag <label>) */
  label: string;
  /** directory that holds the apps */
  dir: string;
  /** host/publish implementation */
  host: Host;
  /** maintained for older code/docs; same value as host.type */
  backend: Host["type"];
}

interface UserBag {
  dir?: string;
  url?: string;
  host?: Host;
}

interface UserConfig {
  defaultBag?: string;
  bags?: Record<string, UserBag>;
}

export interface WebbyConfig {
  defaultBag: string;
  bags: Record<string, Bag>;
  configPath?: string;
  dataDir: string;
}

function expandPath(path: string): string {
  if (path === "~") return homedir();
  if (path.startsWith("~/")) return join(homedir(), path.slice(2));
  if (path.includes("$HOME")) return path.replaceAll("$HOME", homedir());
  return path;
}

function absPath(path: string): string {
  return resolve(expandPath(path));
}

function xdgConfigHome(): string {
  return process.env.XDG_CONFIG_HOME ? absPath(process.env.XDG_CONFIG_HOME) : join(homedir(), ".config");
}

function xdgDataHome(): string {
  return process.env.XDG_DATA_HOME ? absPath(process.env.XDG_DATA_HOME) : join(homedir(), ".local", "share");
}

export function defaultConfigPath(): string {
  return process.env.WEBBY_CONFIG
    ? absPath(process.env.WEBBY_CONFIG)
    : join(xdgConfigHome(), "webby", "config.json");
}

export function defaultDataDir(): string {
  return process.env.WEBBY_DATA_DIR ? absPath(process.env.WEBBY_DATA_DIR) : join(xdgDataHome(), "webby");
}

// Convenience for local development and private setups. A distributed install
// should use $WEBBY_ENV or real environment variables.
function findEnvFile(): string | undefined {
  if (process.env.WEBBY_ENV) return existsSync(process.env.WEBBY_ENV) ? process.env.WEBBY_ENV : undefined;
  const localSecret = join(import.meta.dir, ".env.secret");
  return existsSync(localSecret) ? localSecret : undefined;
}

function loadEnvSecret() {
  const file = findEnvFile();
  if (!file) return;
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

function readUserConfig(path = defaultConfigPath()): UserConfig | undefined {
  if (!existsSync(path)) return undefined;
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    throw new Error(`failed to read ${path}: ${msg}`);
  }
}

function bag(label: string, dir: string, host: Host): Bag {
  return { label, dir: absPath(dir), host, backend: host.type };
}

function builtInBags(dataDir: string): Record<string, Bag> {
  return {
    local: bag("local", join(dataDir, "local"), {
      type: "local",
      url: process.env.WEBBY_LOCAL_URL,
      port: Number(process.env.WEBBY_LOCAL_PORT || 8765),
    }),
    tailnet: bag("tailnet", join(dataDir, "tailnet"), {
      type: "tailscale-serve",
      url: process.env.TAILSCALE_URL,
      path: process.env.TAILSCALE_PATH || "/",
      background: true,
    }),
    funnel: bag("funnel", join(dataDir, "funnel"), {
      type: "tailscale-funnel",
      url: process.env.FUNNEL_URL,
      path: process.env.FUNNEL_PATH || "/",
      background: true,
    }),
    public: bag("public", process.env.PUBLIC_DIR ?? join(dataDir, "public"), {
      type: "cloudflare-pages",
      url: process.env.PUBLIC_URL,
      project: process.env.PUBLIC_PROJECT ?? "webby",
      accountId: process.env.CF_ACCOUNT_ID ?? process.env.CLOUDFLARE_ACCOUNT_ID,
      tokenEnv: process.env.CF_TOKEN_ENV ?? "CLOUDFLARE_API_TOKEN",
      tokenRef: process.env.CF_TOKEN_REF,
      tokenCommand: process.env.CF_TOKEN_COMMAND,
    }),
  };
}

function legacyPrivateBags(bags: Record<string, Bag>) {
  if (process.env.INTERNAL_URL || process.env.INTERNAL_DIR) {
    bags.internal = bag("internal", process.env.INTERNAL_DIR ?? join(import.meta.dir, "internal"), {
      type: (process.env.INTERNAL_HOST_TYPE as Host["type"]) || "caddy",
      url: process.env.INTERNAL_URL,
    } as Host);
  }

  if (process.env.PUBLIC_URL || process.env.PUBLIC_DIR || process.env.PUBLIC_PROJECT) {
    bags.public = bag("public", process.env.PUBLIC_DIR ?? bags.public.dir, {
      type: "cloudflare-pages",
      url: process.env.PUBLIC_URL,
      project: process.env.PUBLIC_PROJECT ?? "webby",
      accountId: process.env.CF_ACCOUNT_ID ?? process.env.CLOUDFLARE_ACCOUNT_ID,
      tokenEnv: process.env.CF_TOKEN_ENV ?? "CLOUDFLARE_API_TOKEN",
      tokenRef: process.env.CF_TOKEN_REF,
      tokenCommand: process.env.CF_TOKEN_COMMAND,
    });
  }
}

function mergeUserConfig(base: Record<string, Bag>, user?: UserConfig): Record<string, Bag> {
  if (!user?.bags) return base;
  const merged = { ...base };
  for (const [label, cfg] of Object.entries(user.bags)) {
    const prev = merged[label];
    const host = cfg.host ?? (cfg.url ? { type: "local", url: cfg.url } : prev?.host) ?? { type: "local" };
    const dir = cfg.dir ?? prev?.dir ?? join(defaultDataDir(), label);
    merged[label] = bag(label, dir, cfg.url && !("url" in host) ? { ...host, url: cfg.url } as Host : host);
  }
  return merged;
}

export function loadConfig(): WebbyConfig {
  loadEnvSecret();
  const dataDir = defaultDataDir();
  const path = defaultConfigPath();
  const user = readUserConfig(path);
  const bags = builtInBags(dataDir);
  legacyPrivateBags(bags);
  const merged = mergeUserConfig(bags, user);
  const defaultBag = process.env.WEBBY_DEFAULT_BAG ?? user?.defaultBag ?? (merged.internal ? "internal" : "local");
  return {
    defaultBag,
    bags: merged,
    configPath: existsSync(path) ? path : undefined,
    dataDir,
  };
}

export function sampleConfig(): string {
  return JSON.stringify({
    defaultBag: "local",
    bags: {
      local: {
        dir: "~/.local/share/webby/local",
        host: { type: "local", port: 8765 },
      },
      tailnet: {
        dir: "~/.local/share/webby/tailnet",
        host: { type: "tailscale-serve", path: "/", background: true },
      },
      funnel: {
        dir: "~/.local/share/webby/funnel",
        host: { type: "tailscale-funnel", path: "/", background: true },
      },
      public: {
        dir: "~/.local/share/webby/public",
        host: {
          type: "cloudflare-pages",
          project: "webby",
          tokenEnv: "CLOUDFLARE_API_TOKEN",
        },
      },
    },
  }, null, 2) + "\n";
}

export { dirname };
