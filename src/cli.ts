#!/usr/bin/env bun
// webby — drop a simple HTML app and serve it.
//   internal (Caddy/html-bag, instant)  |  public (Cloudflare Pages)
// Domains, account id, and the token reference come from .env.secret.

import { parseArgs } from "node:util";
import { stat, lstat, cp, mkdir, rm, readdir, writeFile, symlink, readlink } from "node:fs/promises";
import { existsSync } from "node:fs";
import { basename, extname, join, resolve } from "node:path";
import { BAGS, DEFAULT_BAG, ACCOUNT_ID, CF_TOKEN_REF, type Bag } from "../webby.config.ts";
import { renderIndex, type AppEntry } from "./render.ts";

const API = "https://api.cloudflare.com/client/v4";

function die(msg: string): never {
  console.error(`✗ ${msg}`);
  process.exit(1);
}

function pickBag(opts: { public?: boolean; bag?: string }): Bag {
  const label = opts.public ? "public" : opts.bag ?? DEFAULT_BAG;
  const bag = BAGS[label];
  if (!bag) die(`unknown bag '${label}' (have: ${Object.keys(BAGS).join(", ")})`);
  return bag;
}

// --- staging --------------------------------------------------------------
async function stageApp(
  srcArg: string,
  bag: Bag,
  opts: { name?: string; tmp?: boolean },
): Promise<{ name: string; url: string; isDir: boolean }> {
  const src = resolve(srcArg);
  if (!existsSync(src)) die(`not found: ${src}`);
  const st = await stat(src);
  const isDir = st.isDirectory();

  let name = opts.name ?? (isDir ? basename(src) : basename(src, extname(src)));
  if (!isDir && extname(src).toLowerCase() !== ".html") {
    die("a standalone app must be a .html file (or pass a directory with index.html)");
  }
  if (opts.tmp && !name.startsWith("tmp")) name = `tmp-${name}`;

  await mkdir(bag.dir, { recursive: true });

  if (isDir) {
    if (!existsSync(join(src, "index.html"))) console.warn(`  ! ${name}/ has no index.html`);
    const dest = join(bag.dir, name);
    await rm(dest, { recursive: true, force: true });
    await cp(src, dest, { recursive: true });
    return { name, url: `${bag.url}/${name}/`, isDir: true };
  }
  const dest = join(bag.dir, `${name}.html`);
  await cp(src, dest);
  return { name, url: `${bag.url}/${name}.html`, isDir: false };
}

async function listApps(bag: Bag): Promise<AppEntry[]> {
  if (!existsSync(bag.dir)) return [];
  const entries = await readdir(bag.dir, { withFileTypes: true });
  const apps: AppEntry[] = [];
  for (const e of entries) {
    if (e.name.startsWith(".")) continue;
    // Symlinks (public apps linked into the internal bag) resolve to their target.
    const isDir = e.isSymbolicLink()
      ? await stat(join(bag.dir, e.name)).then((s) => s.isDirectory()).catch(() => false)
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

// Mirror every public app into the internal bag as a relative symlink, so the
// Caddy tools listing shows public apps alongside internal ones. The public
// dir stays the single source of truth — deploy still just pushes public/.
async function syncPublicLinks(): Promise<number> {
  const pub = BAGS.public, internal = BAGS.internal;
  if (!internal || !existsSync(pub.dir)) return 0;
  await mkdir(internal.dir, { recursive: true });
  const pubBase = basename(pub.dir);
  let n = 0;
  for (const e of await readdir(pub.dir, { withFileTypes: true })) {
    if (e.name.startsWith(".") || e.name === "index.html") continue;
    const link = join(internal.dir, e.name);
    const target = `../${pubBase}/${e.name}`; // sibling dirs → resolves on host and in-container
    // Replace only our own symlinks; never clobber a real internal app of the same name.
    if (existsSync(link)) {
      const isLink = await lstat(link).then((s) => s.isSymbolicLink()).catch(() => false);
      if (!isLink) continue;
      if ((await readlink(link).catch(() => "")) === target) continue;
      await rm(link, { force: true });
    }
    await symlink(target, link);
    n++;
  }
  return n;
}

// --- secrets / cloudflare -------------------------------------------------
async function opRead(ref: string): Promise<string> {
  const p = Bun.spawn(["op", "read", ref], { stdout: "pipe", stderr: "pipe" });
  const out = await new Response(p.stdout).text();
  if ((await p.exited) !== 0) {
    die(`op read failed for ${ref}: ${(await new Response(p.stderr).text()).trim()}`);
  }
  return out.trim();
}

async function cf(path: string, token: string, init?: RequestInit) {
  const r = await fetch(`${API}${path}`, {
    ...init,
    headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json", ...(init?.headers ?? {}) },
  });
  return r.json();
}

async function ensureProject(project: string, token: string) {
  const list = await cf(`/accounts/${ACCOUNT_ID}/pages/projects`, token);
  if (list.success && list.result.some((p: { name: string }) => p.name === project)) return;
  console.log(`  creating Pages project '${project}'…`);
  const res = await cf(`/accounts/${ACCOUNT_ID}/pages/projects`, token, {
    method: "POST",
    body: JSON.stringify({ name: project, production_branch: "main" }),
  });
  if (!res.success) die(`project create failed: ${JSON.stringify(res.errors)}`);
}

async function generateIndex(bag: Bag): Promise<AppEntry[]> {
  const apps = await listApps(bag);
  await writeFile(join(bag.dir, "index.html"), renderIndex({ apps, title: "webby" }));
  return apps;
}

async function deploy(bag: Bag) {
  if (bag.backend !== "pages") die(`bag '${bag.label}' is served by ${bag.backend}, nothing to deploy`);
  const linked = await syncPublicLinks();
  if (linked) console.log(`  linked ${linked} public app(s) into the internal bag`);
  const apps = await generateIndex(bag);
  const token = await opRead(CF_TOKEN_REF);
  await ensureProject(bag.project!, token);
  console.log(`  deploying ${apps.length} app(s) to Pages project '${bag.project}'…`);
  const proc = Bun.spawn(
    ["bunx", "wrangler", "pages", "deploy", bag.dir, "--project-name", bag.project!, "--branch", "main", "--commit-dirty=true"],
    { env: { ...process.env, CLOUDFLARE_API_TOKEN: token, CLOUDFLARE_ACCOUNT_ID: ACCOUNT_ID }, stdout: "inherit", stderr: "inherit" },
  );
  if ((await proc.exited) !== 0) die("wrangler deploy failed");
  console.log(`✓ live: ${bag.url}`);
}

// --- commands -------------------------------------------------------------
async function cmdAdd(positionals: string[], opts: any) {
  const path = positionals[0] ?? die("usage: webby add <path> [--name N] [--tmp] [--public]");
  const bag = pickBag(opts);
  const res = await stageApp(path, bag, opts);
  console.log(`✓ ${res.name} → ${bag.label} bag`);
  if (bag.backend === "caddy") {
    console.log(`  live now: ${res.url}`);
  } else {
    await generateIndex(bag);
    console.log(`  staged: ${res.url}  (run \`webby deploy --public\` to publish)`);
  }
}

async function cmdPub(positionals: string[], opts: any) {
  const path = positionals[0] ?? die("usage: webby pub <path> [--name N] [--tmp]");
  const bag = BAGS.public;
  const res = await stageApp(path, bag, opts);
  console.log(`✓ ${res.name} → public bag`);
  await deploy(bag);
}

async function cmdDeploy(opts: any) {
  await deploy(pickBag({ ...opts, public: opts.public ?? opts.bag === undefined }));
}

async function cmdLs(opts: any) {
  const bag = pickBag(opts);
  const apps = await listApps(bag);
  if (!apps.length) return console.log(`(${bag.label} bag is empty)`);
  console.log(`${bag.label} bag — ${bag.url}`);
  for (const a of apps) {
    const tag = a.tmp ? " ·tmp" : "";
    console.log(`  ${a.name}${a.isDir ? "/" : ".html"}${tag}`);
  }
}

async function cmdRm(positionals: string[], opts: any) {
  const name = positionals[0] ?? die("usage: webby rm <name> [--public]");
  const bag = pickBag(opts);
  const dir = join(bag.dir, name);
  const file = join(bag.dir, name.endsWith(".html") ? name : `${name}.html`);
  let target = "";
  if (existsSync(dir) && (await stat(dir)).isDirectory()) target = dir;
  else if (existsSync(file)) target = file;
  else die(`no app named '${name}' in ${bag.label} bag`);
  await rm(target, { recursive: true, force: true });
  console.log(`✓ removed ${name} from ${bag.label} bag`);
  if (bag.backend === "pages") {
    // Drop the symlink we mirrored into the internal bag, if any.
    const linkName = basename(target);
    const link = join(BAGS.internal!.dir, linkName);
    if (existsSync(link) && (await lstat(link)).isSymbolicLink()) await rm(link, { force: true });
    console.log(`  run \`webby deploy --public\` to update the live site`);
  }
}

async function cmdOpen(positionals: string[], opts: any) {
  const name = positionals[0] ?? die("usage: webby open <name> [--public]");
  const bag = pickBag(opts);
  const url = `${bag.url}/${name.replace(/\/$/, "")}${name.endsWith(".html") ? "" : "/"}`;
  console.log(url);
  Bun.spawn(["xdg-open", url], { stdout: "ignore", stderr: "ignore" }).exited.catch(() => {});
}

async function cmdDomain(positionals: string[]) {
  const host = positionals[0] ?? die("usage: webby domain <hostname>");
  const bag = BAGS.public;
  const token = await opRead(CF_TOKEN_REF);
  const res = await cf(`/accounts/${ACCOUNT_ID}/pages/projects/${bag.project}/domains`, token, {
    method: "POST",
    body: JSON.stringify({ name: host }),
  });
  if (!res.success) die(`attach domain failed: ${JSON.stringify(res.errors)}`);
  console.log(`✓ attached ${host} to Pages project '${bag.project}' (cert/DNS provisioning by Cloudflare)`);
}

function cmdWhere() {
  console.log("webby bags — drop an app into one of these dirs:\n");
  for (const bag of Object.values(BAGS)) {
    console.log(`  ${bag.label.padEnd(9)} ${bag.dir}`);
    console.log(`  ${" ".repeat(9)} → ${bag.url}  (${bag.backend}${bag.project ? `: ${bag.project}` : ""})`);
  }
  console.log(`\n  default bag: ${DEFAULT_BAG}`);
  console.log(`  public apps are mirrored into the internal bag as symlinks (the internal listing shows both).`);
}

const HELP = `webby — drop a simple HTML app and serve it

  webby add <path> [--name N] [--tmp] [--public]   stage an app into a bag
  webby pub <path> [--name N] [--tmp]              add to public bag + deploy
  webby deploy [--public]                          regenerate index + deploy (pages bags)
  webby ls   [--public | --bag <name>]             list apps in a bag
  webby rm   <name> [--public]                     remove an app
  webby open <name> [--public]                     print/open an app URL
  webby domain <hostname>                          attach a custom domain to the public bag
  webby where                                      print bag directories + URLs

Bags: ${Object.values(BAGS).map((b) => `${b.label} (${b.backend} → ${b.url})`).join(", ")}
Default bag: ${DEFAULT_BAG}. <path> is a .html file or a directory with index.html.`;

async function main() {
  const { values, positionals } = parseArgs({
    args: Bun.argv.slice(2),
    allowPositionals: true,
    options: {
      name: { type: "string" },
      tmp: { type: "boolean", default: false },
      public: { type: "boolean", short: "p", default: false },
      bag: { type: "string" },
      help: { type: "boolean", short: "h", default: false },
    },
  });

  const [cmd, ...rest] = positionals;
  if (values.help || !cmd) return console.log(HELP);

  switch (cmd) {
    case "add": return cmdAdd(rest, values);
    case "pub": case "publish": return cmdPub(rest, values);
    case "deploy": return cmdDeploy(values);
    case "ls": case "list": return cmdLs(values);
    case "rm": case "remove": return cmdRm(rest, values);
    case "open": return cmdOpen(rest, values);
    case "domain": return cmdDomain(rest);
    case "where": case "paths": return cmdWhere();
    default: die(`unknown command '${cmd}'\n\n${HELP}`);
  }
}

main();
