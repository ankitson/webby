use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("webby-{name}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_webby"))
}

fn write_app(dir: &Path) {
    fs::create_dir_all(dir.join("app")).unwrap();
    fs::write(
        dir.join("app").join("index.html"),
        "<!doctype html><h1>ok</h1>\n",
    )
    .unwrap();
}

fn write_named_app(root: &Path, name: &str) {
    fs::create_dir_all(root.join(name)).unwrap();
    fs::write(
        root.join(name).join("index.html"),
        format!("<!doctype html><h1>{name}</h1>\n"),
    )
    .unwrap();
}

fn write_app_with_metadata(dir: &Path) {
    write_named_app(dir, "source-app");
    fs::create_dir_all(dir.join("source-app").join(".git")).unwrap();
    fs::create_dir_all(dir.join("source-app").join(".wrangler")).unwrap();
    fs::create_dir_all(dir.join("source-app").join("logs")).unwrap();
    fs::write(dir.join("source-app").join(".git").join("HEAD"), "secret").unwrap();
    fs::write(
        dir.join("source-app").join(".wrangler").join("pages.json"),
        "{}",
    )
    .unwrap();
    fs::write(dir.join("source-app").join("logs").join("run.log"), "log").unwrap();
}

fn write_config(root: &Path, body: &str) -> PathBuf {
    let config = root.join("config.json");
    fs::write(&config, body).unwrap();
    config
}

fn webby(root: &Path, config: &Path) -> Command {
    let mut cmd = Command::new(bin());
    cmd.env_clear()
        .env("HOME", root)
        .env("WEBBY_CONFIG", config)
        .env("WEBBY_DATA_DIR", root.join("data"))
        .env("WEBBY_ENV", root.join("missing.env"));
    cmd
}

fn fake_exe(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    fs::set_permissions(path, perms).unwrap();
}

#[test]
fn deploy_local_and_caddy_generate_indexes_without_external_commands() {
    let tmp = TestDir::new("local-caddy");
    let local = tmp.path().join("local");
    let caddy = tmp.path().join("caddy");
    write_named_app(&local, "app");
    write_named_app(&local, "other");
    fs::create_dir_all(local.join("webby-previews")).unwrap();
    fs::write(local.join("webby-previews").join("app.jpg"), "jpeg").unwrap();
    write_app(&caddy);
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "defaultBag": "local",
              "bags": {{
                "local": {{ "dir": "{}", "host": {{ "type": "local", "port": 7777 }} }},
                "caddy": {{ "dir": "{}", "host": {{ "type": "caddy", "url": "https://caddy.example" }} }}
              }}
            }}"#,
            local.display(),
            caddy.display()
        ),
    );

    let local_out = webby(tmp.path(), &config)
        .args(["deploy", "-b", "local"])
        .output()
        .unwrap();
    assert!(
        local_out.status.success(),
        "{}",
        String::from_utf8_lossy(&local_out.stderr)
    );
    assert!(local.join("index.html").exists());
    assert!(String::from_utf8_lossy(&local_out.stdout).contains("http://localhost:7777"));
    let local_manifest = fs::read_to_string(local.join("webby-cards.json")).unwrap();
    assert!(!local_manifest.contains("\"id\": \"webby-previews\""));

    let caddy_out = webby(tmp.path(), &config)
        .args(["deploy", "-b", "caddy"])
        .output()
        .unwrap();
    assert!(
        caddy_out.status.success(),
        "{}",
        String::from_utf8_lossy(&caddy_out.stderr)
    );
    assert!(caddy.join("index.html").exists());
    assert!(String::from_utf8_lossy(&caddy_out.stdout).contains("https://caddy.example"));
}

#[test]
fn no_index_bag_writes_card_manifest_without_page() {
    let tmp = TestDir::new("no-index");
    let caddy = tmp.path().join("caddy");
    write_named_app(&caddy, "app");
    fs::write(caddy.join("index.html"), "old index").unwrap();
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "bags": {{
                "caddy": {{
                  "dir": "{}",
                  "noIndex": true,
                  "host": {{ "type": "caddy", "url": "https://caddy.example" }}
                }}
              }}
            }}"#,
            caddy.display()
        ),
    );

    let out = webby(tmp.path(), &config)
        .args(["deploy", "-b", "caddy"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(caddy.join("webby-cards.json").exists());
    assert!(caddy.join("webby-card-grid.js").exists());
    assert!(!caddy.join("index.html").exists());
    let manifest = fs::read_to_string(caddy.join("webby-cards.json")).unwrap();
    assert!(manifest.contains("\"id\": \"app\""));
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .contains(&format!("generated: {}/webby-cards.json", caddy.display()))
    );
}

#[test]
fn generated_manifest_uses_app_owned_webby_metadata() {
    let tmp = TestDir::new("app-metadata");
    let local = tmp.path().join("local");
    fs::create_dir_all(&local).unwrap();
    fs::write(
        local.join("report.html"),
        r#"<!doctype html>
<html>
<head>
  <title>Fallback report title</title>
  <script type="application/webby+json">
  {
    "title": "Network Report",
    "description": "A self-contained report.",
    "properties": {
      "category": "Documents",
      "priority": 3
    }
  }
  </script>
</head>
<body>ok</body>
</html>"#,
    )
    .unwrap();
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "defaultBag": "local",
              "bags": {{
                "local": {{ "dir": "{}", "host": {{ "type": "local", "port": 7777 }} }}
              }}
            }}"#,
            local.display()
        ),
    );

    let out = webby(tmp.path(), &config)
        .args(["deploy", "-b", "local"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let manifest = fs::read_to_string(local.join("webby-cards.json")).unwrap();
    assert!(manifest.contains("\"title\": \"Network Report\""));
    assert!(manifest.contains("\"description\": \"A self-contained report.\""));
    assert!(manifest.contains("\"category\": \"Documents\""));
    assert!(manifest.contains("\"properties\": {"));
    assert!(manifest.contains("\"priority\": 3"));
}

#[test]
fn add_can_write_metadata_properties_into_staged_app() {
    let tmp = TestDir::new("add-metadata-flags");
    let source = tmp.path().join("source");
    let local = tmp.path().join("local");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("report.html"),
        "<!doctype html><html><head><title>Old title</title></head><body>ok</body></html>",
    )
    .unwrap();
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "defaultBag": "local",
              "bags": {{
                "local": {{ "dir": "{}", "host": {{ "type": "local", "port": 7777 }} }}
              }}
            }}"#,
            local.display()
        ),
    );

    let out = webby(tmp.path(), &config)
        .args([
            "add",
            source.join("report.html").to_str().unwrap(),
            "--title",
            "Network Report",
            "--description",
            "Self-contained metadata",
            "--property",
            "category=Documents",
            "--property",
            "kind=report",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let staged = fs::read_to_string(local.join("report.html")).unwrap();
    assert!(staged.contains("application/webby+json"));
    assert!(staged.contains("\"category\": \"Documents\""));
    assert!(staged.contains("\"kind\": \"report\""));

    let manifest = fs::read_to_string(local.join("webby-cards.json")).unwrap();
    assert!(manifest.contains("\"title\": \"Network Report\""));
    assert!(manifest.contains("\"description\": \"Self-contained metadata\""));
    assert!(manifest.contains("\"category\": \"Documents\""));
    assert!(manifest.contains("\"kind\": \"report\""));
}

#[test]
fn deploy_no_index_flag_skips_root_index_without_config_mode() {
    let tmp = TestDir::new("no-index-flag");
    let caddy = tmp.path().join("caddy");
    write_named_app(&caddy, "app");
    fs::write(caddy.join("index.html"), "old index").unwrap();
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "bags": {{
                "caddy": {{
                  "dir": "{}",
                  "host": {{ "type": "caddy", "url": "https://caddy.example" }}
                }}
              }}
            }}"#,
            caddy.display()
        ),
    );

    let out = webby(tmp.path(), &config)
        .args(["deploy", "-b", "caddy", "--no-index"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(caddy.join("webby-cards.json").exists());
    assert!(caddy.join("webby-card-grid.js").exists());
    assert!(!caddy.join("index.html").exists());
    assert!(
        String::from_utf8_lossy(&out.stdout)
            .contains(&format!("generated: {}/webby-cards.json", caddy.display()))
    );
}

#[test]
fn add_directory_skips_local_metadata_dirs() {
    let tmp = TestDir::new("copy-skip");
    let source = tmp.path().join("source");
    let public = tmp.path().join("public");
    write_app_with_metadata(&source);
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "bags": {{
                "public": {{
                  "dir": "{}",
                  "host": {{ "type": "local", "port": 7777 }}
                }}
              }}
            }}"#,
            public.display()
        ),
    );

    let out = webby(tmp.path(), &config)
        .args([
            "add",
            source.join("source-app").to_str().unwrap(),
            "-b",
            "public",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let staged = public.join("source-app");
    assert!(staged.join("index.html").exists());
    assert!(!staged.join(".git").exists());
    assert!(!staged.join(".wrangler").exists());
    assert!(!staged.join("logs").exists());
}

#[test]
fn preview_uses_shot_scraper_via_uvx() {
    let tmp = TestDir::new("preview");
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let capture = tmp.path().join("uvx.log");
    fake_exe(
        &bin_dir,
        "uvx",
        r#"#!/bin/sh
echo "uvx $*" >> "$WEBBY_CAPTURE"
out=""
prev=""
for arg in "$@"; do
  if [ "$prev" = "--output" ]; then out="$arg"; fi
  prev="$arg"
done
if [ -z "$out" ]; then exit 2; fi
printf "jpeg" > "$out"
exit 0
"#,
    );

    let local = tmp.path().join("local");
    write_app(&local);
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "defaultBag": "local",
              "bags": {{
                "local": {{ "dir": "{}", "host": {{ "type": "local", "port": 7777 }} }}
              }}
            }}"#,
            local.display()
        ),
    );

    let out = webby(tmp.path(), &config)
        .env("PATH", bin_dir)
        .env("WEBBY_CAPTURE", &capture)
        .args([
            "preview",
            "app",
            "-b",
            "local",
            "--force",
            "--width",
            "640",
            "--height",
            "360",
            "--timeout-secs",
            "2",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(local.join("webby-previews").join("app.jpg").exists());
    assert!(!local.join("webby-previews").join("other.jpg").exists());

    let log = fs::read_to_string(capture).unwrap();
    assert!(log.contains("uvx shot-scraper shot"));
    assert_eq!(log.lines().count(), 1);
    assert!(log.contains("--width 640 --height 360"));
    assert!(log.contains("--wait 2000"));
    assert!(log.contains("--timeout 2000"));
}

#[test]
fn deploy_tailscale_providers_call_expected_subcommands() {
    let tmp = TestDir::new("tailscale");
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let capture = tmp.path().join("commands.log");
    fake_exe(
        &bin_dir,
        "tailscale",
        r#"#!/bin/sh
echo "tailscale $*" >> "$WEBBY_CAPTURE"
exit 0
"#,
    );

    let tailnet = tmp.path().join("tailnet");
    let funnel = tmp.path().join("funnel");
    write_app(&tailnet);
    write_app(&funnel);
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "bags": {{
                "tailnet": {{ "dir": "{}", "host": {{ "type": "tailscale-serve", "url": "https://tail.example", "path": "/webby", "background": true }} }},
                "funnel": {{ "dir": "{}", "host": {{ "type": "tailscale-funnel", "url": "https://funnel.example", "path": "/demo", "background": true }} }}
              }}
            }}"#,
            tailnet.display(),
            funnel.display()
        ),
    );

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        env::var("PATH").unwrap_or_default()
    );
    for bag in ["tailnet", "funnel"] {
        let out = webby(tmp.path(), &config)
            .env("PATH", &path)
            .env("WEBBY_CAPTURE", &capture)
            .args(["deploy", "-b", bag])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let log = fs::read_to_string(capture).unwrap();
    assert!(log.contains(&format!(
        "tailscale serve --bg --set-path /webby {}",
        tailnet.display()
    )));
    assert!(log.contains(&format!(
        "tailscale funnel --bg --set-path /demo {}",
        funnel.display()
    )));
}

#[test]
fn deploy_cloudflare_pages_calls_wrangler_with_env() {
    let tmp = TestDir::new("cloudflare");
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let capture = tmp.path().join("wrangler.log");
    fake_exe(
        &bin_dir,
        "wrangler",
        r#"#!/bin/sh
echo "wrangler $*" >> "$WEBBY_CAPTURE"
echo "account=$CLOUDFLARE_ACCOUNT_ID token=$CLOUDFLARE_API_TOKEN" >> "$WEBBY_CAPTURE"
exit 0
"#,
    );

    let public = tmp.path().join("public");
    write_app(&public);
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "bags": {{
                "public": {{
                  "dir": "{}",
                  "host": {{
                    "type": "cloudflare-pages",
                    "url": "https://public.example",
                    "project": "mini",
                    "accountId": "acct_123",
                    "tokenEnv": "WEBBY_TEST_CF_TOKEN",
                    "branch": "preview"
                  }}
                }}
              }}
            }}"#,
            public.display()
        ),
    );

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        env::var("PATH").unwrap_or_default()
    );
    let out = webby(tmp.path(), &config)
        .env("PATH", path)
        .env("WEBBY_CAPTURE", &capture)
        .env("WEBBY_TEST_CF_TOKEN", "secret-token")
        .args(["deploy", "-b", "public"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(public.join("index.html").exists());

    let log = fs::read_to_string(capture).unwrap();
    assert!(log.contains(&format!(
        "wrangler pages deploy {} --project-name mini --branch preview --commit-dirty=true",
        public.display()
    )));
    assert!(log.contains("account=acct_123 token=secret-token"));
}

#[test]
fn deploy_cloudflare_pages_uses_npx_fallback_when_wrangler_is_missing() {
    let tmp = TestDir::new("cloudflare-npx");
    let bin_dir = tmp.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let capture = tmp.path().join("npx.log");
    fake_exe(
        &bin_dir,
        "npx",
        r#"#!/bin/sh
echo "npx $*" >> "$WEBBY_CAPTURE"
exit 0
"#,
    );

    let public = tmp.path().join("public");
    write_app(&public);
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "bags": {{
                "public": {{
                  "dir": "{}",
                  "host": {{
                    "type": "cloudflare-pages",
                    "url": "https://public.example",
                    "project": "mini",
                    "tokenEnv": "WEBBY_TEST_CF_TOKEN"
                  }}
                }}
              }}
            }}"#,
            public.display()
        ),
    );

    let out = webby(tmp.path(), &config)
        .env("PATH", bin_dir)
        .env("WEBBY_CAPTURE", &capture)
        .env("WEBBY_TEST_CF_TOKEN", "secret-token")
        .args(["deploy", "-b", "public"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(public.join("index.html").exists());

    let log = fs::read_to_string(capture).unwrap();
    assert!(log.contains(&format!(
        "npx --yes wrangler pages deploy {} --project-name mini --branch main --commit-dirty=true",
        public.display()
    )));
}

#[test]
fn deploy_command_provider_expands_template() {
    let tmp = TestDir::new("command");
    let command_bag = tmp.path().join("command");
    let capture = tmp.path().join("command.log");
    write_app(&command_bag);
    let config = write_config(
        tmp.path(),
        &format!(
            r#"{{
              "bags": {{
                "cmd": {{
                  "dir": "{}",
                  "host": {{
                    "type": "command",
                    "url": "https://cmd.example",
                    "deploy": "echo deploy {{label}} {{dir}} {{url}} >> {}"
                  }}
                }}
              }}
            }}"#,
            command_bag.display(),
            capture.display()
        ),
    );

    let out = webby(tmp.path(), &config)
        .args(["deploy", "-b", "cmd"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(command_bag.join("index.html").exists());

    let log = fs::read_to_string(capture).unwrap();
    assert!(log.contains(&format!(
        "deploy cmd {} https://cmd.example",
        command_bag.display()
    )));
}
