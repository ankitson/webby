import { existsSync } from "node:fs";
import { mkdir, readdir, stat } from "node:fs/promises";
import { basename, extname, join, normalize, relative, resolve } from "node:path";
import type { Bag, Host } from "../webby.config.ts";
import { renderIndex, type AppEntry } from "./render.ts";

const API = "https://api.cloudflare.com/client/v4";

export function die(msg: string): never {
  console.error(`✗ ${msg}`);
  process.exit(1);
}

export function assertCloudflareHost(host: Host): Extract<Host, { type: "cloudflare-pages" }> {
  if (host.type !== "cloudflare-pages") die("this command requires a Cloudflare Pages bag");
  return host;
}

export function isDurablePublic(host: Host): boolean {
  return host.type === "cloudflare-pages";
}

export function localUrl(bag: Bag, port?: number): string {
  const host = bag.host.type === "local" ? bag.host : undefined;
  const p = port ?? host?.port ?? 8765;
  return host?.url ?? `http://localhost:${p}`;
}

async function tailscaleHostname(): Promise<string | undefined> {
  const proc = Bun.spawn(["tailscale", "status", "--json"], { stdout: "pipe", stderr: "pipe" });
  const out = await new Response(proc.stdout).text();
  if ((await proc.exited) !== 0) return undefined;
  try {
    const data = JSON.parse(out);
    const dns = data?.Self?.DNSName;
    return typeof dns === "string" ? dns.replace(/\.$/, "") : undefined;
  } catch {
    return undefined;
  }
}

function withPath(base: string, path?: string): string {
  const clean = path && path !== "/" ? `/${path.replace(/^\/+|\/+$/g, "")}` : "";
  return `${base.replace(/\/+$/g, "")}${clean}`;
}

export async function bagBaseUrl(bag: Bag, port?: number): Promise<string> {
  const host = bag.host;
  if (host.type === "local") return localUrl(bag, port);
  if (host.url) return withPath(host.url, "path" in host ? host.path : undefined);
  if (host.type === "tailscale-serve" || host.type === "tailscale-funnel") {
    const dns = await tailscaleHostname();
    if (dns) return withPath(`https://${dns}`, host.path);
  }
  return "(url unknown until host is configured)";
}

export function appUrl(baseUrl: string, name: string, isDir: boolean): string {
  if (baseUrl.startsWith("(")) return baseUrl;
  return `${baseUrl.replace(/\/+$/g, "")}/${name}${isDir ? "/" : ".html"}`;
}

export async function listApps(bag: Bag): Promise<AppEntry[]> {
  if (!existsSync(bag.dir)) return [];
  const entries = await readdir(bag.dir, { withFileTypes: true });
  const apps: AppEntry[] = [];
  for (const e of entries) {
    if (e.name.startsWith(".")) continue;
    const full = join(bag.dir, e.name);
    const isDir = e.isSymbolicLink()
      ? await stat(full).then((s) => s.isDirectory()).catch(() => false)
      : e.isDirectory();
    if (isDir) {
      apps.push({ name: e.name, isDir: true, href: `./${e.name}/`, tmp: e.name.startsWith("tmp") });
    } else if (e.name.toLowerCase().endsWith(".html") && e.name !== "index.html") {
      const base = e.name.replace(/\.html$/i, "");
      apps.push({ name: base, isDir: false, href: `./${e.name}`, tmp: base.startsWith("tmp") });
    }
  }
  return apps.sort((a, b) => a.name.localeCompare(b.name));
}

export async function generateIndex(bag: Bag): Promise<AppEntry[]> {
  await mkdir(bag.dir, { recursive: true });
  const apps = await listApps(bag);
  await Bun.write(join(bag.dir, "index.html"), renderIndex({ apps, title: "webby" }));
  return apps;
}

export async function runCommandTemplate(command: string, bag: Bag) {
  const replacements: Record<string, string> = {
    "{dir}": bag.dir,
    "{label}": bag.label,
    "{url}": await bagBaseUrl(bag),
  };
  const expanded = Object.entries(replacements).reduce((cmd, [k, v]) => cmd.replaceAll(k, v), command);
  const proc = Bun.spawn(["sh", "-c", expanded], { stdout: "inherit", stderr: "inherit" });
  if ((await proc.exited) !== 0) die(`command failed: ${expanded}`);
}

export async function activateBag(bag: Bag, opts: { port?: number } = {}) {
  const host = bag.host;
  await generateIndex(bag);

  if (host.type === "local") {
    const port = opts.port ?? host.port ?? 8765;
    console.log(`  serving ${bag.dir}`);
    console.log(`✓ live: ${localUrl(bag, port)}`);
    await serveStatic(bag, port);
    return;
  }

  if (host.type === "tailscale-serve" || host.type === "tailscale-funnel") {
    const args = [
      "tailscale",
      host.type === "tailscale-funnel" ? "funnel" : "serve",
      ...(host.background ?? true ? ["--bg"] : []),
      ...(host.path && host.path !== "/" ? ["--set-path", host.path] : []),
      bag.dir,
    ];
    const proc = Bun.spawn(args, { stdout: "inherit", stderr: "inherit" });
    if ((await proc.exited) !== 0) die(`${args.slice(0, 2).join(" ")} failed`);
    console.log(`✓ live: ${await bagBaseUrl(bag)}`);
    return;
  }

  if (host.type === "cloudflare-pages") {
    await deployCloudflarePages(bag, host);
    return;
  }

  if (host.type === "command") {
    if (!host.deploy) die(`bag '${bag.label}' has no deploy command`);
    await runCommandTemplate(host.deploy, bag);
    console.log(`✓ live: ${await bagBaseUrl(bag)}`);
    return;
  }

  if (host.type === "caddy") {
    console.log(`✓ live: ${await bagBaseUrl(bag)}`);
    return;
  }
}

export async function tokenFromCloudflareHost(host: Extract<Host, { type: "cloudflare-pages" }>): Promise<string> {
  const envKey = host.tokenEnv ?? "CLOUDFLARE_API_TOKEN";
  const fromEnv = process.env[envKey] ?? process.env.CF_API_TOKEN ?? process.env.CF_TOKEN;
  if (fromEnv) return fromEnv;

  if (host.tokenRef) {
    const p = Bun.spawn(["op", "read", host.tokenRef], { stdout: "pipe", stderr: "pipe" });
    const out = await new Response(p.stdout).text();
    if ((await p.exited) !== 0) {
      die(`op read failed for ${host.tokenRef}: ${(await new Response(p.stderr).text()).trim()}`);
    }
    return out.trim();
  }

  if (host.tokenCommand) {
    const p = Bun.spawn(["sh", "-c", host.tokenCommand], { stdout: "pipe", stderr: "pipe" });
    const out = await new Response(p.stdout).text();
    if ((await p.exited) !== 0) {
      die(`token command failed: ${(await new Response(p.stderr).text()).trim()}`);
    }
    return out.trim();
  }

  die(`missing Cloudflare token. Set ${envKey}, CF_TOKEN_REF, or tokenCommand in config.`);
}

async function cf(accountId: string, path: string, token: string, init?: RequestInit) {
  const r = await fetch(`${API}${path}`, {
    ...init,
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json", ...(init?.headers ?? {}) },
  });
  return r.json();
}

async function ensureProject(accountId: string, project: string, token: string) {
  const list = await cf(accountId, `/accounts/${accountId}/pages/projects`, token);
  if (list.success && list.result.some((p: { name: string }) => p.name === project)) return;
  console.log(`  creating Pages project '${project}'...`);
  const res = await cf(accountId, `/accounts/${accountId}/pages/projects`, token, {
    method: "POST",
    body: JSON.stringify({ name: project, production_branch: "main" }),
  });
  if (!res.success) die(`project create failed: ${JSON.stringify(res.errors)}`);
}

export async function deployCloudflarePages(bag: Bag, host: Extract<Host, { type: "cloudflare-pages" }>) {
  const apps = await generateIndex(bag);
  const accountId = host.accountId ?? process.env.CLOUDFLARE_ACCOUNT_ID ?? process.env.CF_ACCOUNT_ID;
  if (!accountId) die("missing Cloudflare account id. Set CLOUDFLARE_ACCOUNT_ID or host.accountId.");
  const project = host.project ?? "webby";
  const token = await tokenFromCloudflareHost(host);
  await ensureProject(accountId, project, token);
  console.log(`  deploying ${apps.length} app(s) to Pages project '${project}'...`);
  const proc = Bun.spawn(
    ["bunx", "wrangler", "pages", "deploy", bag.dir, "--project-name", project, "--branch", "main", "--commit-dirty=true"],
    { env: { ...process.env, CLOUDFLARE_API_TOKEN: token, CLOUDFLARE_ACCOUNT_ID: accountId }, stdout: "inherit", stderr: "inherit" },
  );
  if ((await proc.exited) !== 0) die("wrangler deploy failed");
  console.log(`✓ live: ${await bagBaseUrl(bag)}`);
}

function contentType(path: string): string {
  switch (extname(path).toLowerCase()) {
    case ".html": return "text/html; charset=utf-8";
    case ".css": return "text/css; charset=utf-8";
    case ".js": return "text/javascript; charset=utf-8";
    case ".json": return "application/json; charset=utf-8";
    case ".svg": return "image/svg+xml";
    case ".png": return "image/png";
    case ".jpg":
    case ".jpeg": return "image/jpeg";
    case ".gif": return "image/gif";
    case ".webp": return "image/webp";
    case ".ico": return "image/x-icon";
    case ".wasm": return "application/wasm";
    default: return "application/octet-stream";
  }
}

async function serveFile(path: string): Promise<Response> {
  const s = await stat(path).catch(() => undefined);
  if (!s) return new Response("Not found\n", { status: 404 });
  const file = s.isDirectory() ? join(path, "index.html") : path;
  const fileStat = await stat(file).catch(() => undefined);
  if (!fileStat?.isFile()) return new Response("Not found\n", { status: 404 });
  return new Response(Bun.file(file), {
    headers: { "Content-Type": contentType(file) },
  });
}

export async function serveStatic(bag: Bag, port: number) {
  await generateIndex(bag);
  const root = resolve(bag.dir);
  const server = Bun.serve({
    port,
    async fetch(req) {
      const url = new URL(req.url);
      const decoded = decodeURIComponent(url.pathname);
      const target = normalize(join(root, decoded));
      const rel = relative(root, target);
      if (rel === ".." || rel.startsWith("../") || rel.startsWith("..\\")) {
        return new Response("Forbidden\n", { status: 403 });
      }
      return serveFile(target);
    },
  });
  process.on("SIGINT", () => {
    server.stop();
    process.exit(0);
  });
  await new Promise(() => {});
}

export function displayBagHost(host: Host): string {
  if (host.type === "cloudflare-pages") return `${host.type}${host.project ? `: ${host.project}` : ""}`;
  if (host.type === "local") return `${host.type}${host.port ? `: ${host.port}` : ""}`;
  return host.type;
}

export function displayEntryName(app: AppEntry): string {
  return `${app.name}${app.isDir ? "/" : ".html"}${app.tmp ? " ·tmp" : ""}`;
}

export function inferAppName(src: string, isDir: boolean): string {
  return isDir ? basename(src) : basename(src, extname(src));
}
