use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use minijinja::{Environment, context};
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::{Result, err};

const GENERATED_MARKER: &str = ".webby-docs";

#[derive(Clone, Debug)]
pub struct DocsOptions {
    pub name: Option<String>,
    pub tmp: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub properties: BTreeMap<String, Value>,
    pub depth: usize,
    pub max_asset_size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocsBuildResult {
    pub name: String,
    pub title: String,
    pub page_count: usize,
    pub app_dir: PathBuf,
}

#[derive(Clone, Debug)]
struct Page {
    source: PathBuf,
    source_rel: PathBuf,
    output_rel: PathBuf,
    title: String,
    description: Option<String>,
    frontmatter: Frontmatter,
    body: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct Frontmatter {
    title: Option<String>,
    name: Option<String>,
    description: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    resource: Option<String>,
    tags: Vec<String>,
    timestamp: Option<String>,
}

#[derive(Clone, Debug)]
struct Asset {
    source: PathBuf,
    output_rel: PathBuf,
}

#[derive(Clone, Debug)]
struct LinkMaps {
    pages: HashMap<PathBuf, PathBuf>,
    assets: HashMap<PathBuf, PathBuf>,
}

pub fn build_docs_app(
    src_arg: &Path,
    output_root: &Path,
    options: &DocsOptions,
) -> Result<DocsBuildResult> {
    if !src_arg.is_dir() {
        return Err(err(format!(
            "docs source must be a directory: {}",
            src_arg.display()
        )));
    }
    let root = fs::canonicalize(src_arg)?;
    let mut app_name = options
        .name
        .clone()
        .unwrap_or_else(|| title_slug(&root.file_name().and_then(OsStr::to_str).unwrap_or("docs")));
    if app_name.is_empty() {
        app_name = "docs".to_string();
    }
    if options.tmp && !app_name.starts_with("tmp") {
        app_name = format!("tmp-{app_name}");
    }
    validate_app_name(&app_name)?;

    let markdown_files = discover_markdown(&root, options.depth)?;
    if markdown_files.is_empty() {
        return Err(err(format!(
            "no markdown files found under {} within depth {}",
            root.display(),
            options.depth
        )));
    }

    let output_dir = output_root.join(&app_name);
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir)?;
    }
    fs::create_dir_all(&output_dir)?;
    fs::write(
        output_dir.join(GENERATED_MARKER),
        format!("source={}\n", root.display()),
    )?;

    let mut pages = build_pages(&root, markdown_files)?;
    let assets = discover_linked_assets(&root, &pages, options.max_asset_size)?;
    assign_page_outputs(&mut pages)?;
    let link_maps = LinkMaps {
        pages: pages
            .iter()
            .map(|page| (page.source.clone(), page.output_rel.clone()))
            .collect(),
        assets: assets
            .iter()
            .map(|asset| (asset.source.clone(), asset.output_rel.clone()))
            .collect(),
    };

    for asset in &assets {
        copy_asset(asset, &output_dir)?;
    }

    let site_title = options
        .title
        .clone()
        .or_else(|| root_index(&pages).and_then(|page| page.frontmatter.title.clone()))
        .unwrap_or_else(|| {
            title_from_text(root.file_name().and_then(OsStr::to_str).unwrap_or("Docs"))
        });
    let site_description = options
        .description
        .clone()
        .or_else(|| root_index(&pages).and_then(|page| page.description.clone()));
    let mut properties = options.properties.clone();
    properties
        .entry("kind".to_string())
        .or_insert_with(|| Value::String("markdown-docs".to_string()));

    if !pages
        .iter()
        .any(|page| page.output_rel == Path::new("index.html"))
    {
        write_synthetic_home(
            &output_dir,
            &site_title,
            site_description.as_deref(),
            &properties,
            &pages,
        )?;
    }

    for page in &pages {
        let nav_html = render_nav(&pages, &page.output_rel);
        write_page(
            &output_dir,
            page,
            &link_maps,
            &site_title,
            &site_description,
            &properties,
            &nav_html,
        )?;
    }

    Ok(DocsBuildResult {
        name: app_name,
        title: site_title,
        page_count: pages.len(),
        app_dir: output_dir,
    })
}

fn discover_markdown(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut queue = VecDeque::from([root.to_path_buf()]);
    while let Some(dir) = queue.pop_front() {
        let rel = dir.strip_prefix(root).unwrap_or(Path::new(""));
        let depth = rel.components().count();
        let mut entries = fs::read_dir(&dir)?.collect::<std::result::Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name();
            if should_skip_name(&name) {
                continue;
            }
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                if depth < max_depth {
                    queue.push_back(path);
                }
            } else if file_type.is_file()
                && path
                    .extension()
                    .and_then(OsStr::to_str)
                    .map(|extension| extension.eq_ignore_ascii_case("md"))
                    == Some(true)
            {
                files.push(fs::canonicalize(path)?);
            }
        }
    }
    files.sort_by_key(|path| path.strip_prefix(root).unwrap_or(path).to_path_buf());
    Ok(files)
}

fn build_pages(root: &Path, markdown_files: Vec<PathBuf>) -> Result<Vec<Page>> {
    markdown_files
        .into_iter()
        .map(|source| {
            let source_rel = source
                .strip_prefix(root)
                .map_err(|_| {
                    err(format!(
                        "{} is outside {}",
                        source.display(),
                        root.display()
                    ))
                })?
                .to_path_buf();
            let text = fs::read_to_string(&source)
                .map_err(|e| err(format!("failed to read {}: {e}", source.display())))?;
            let (frontmatter, body) = split_frontmatter(&text);
            let title = page_title(&source_rel, &frontmatter, body);
            Ok(Page {
                source,
                source_rel,
                output_rel: PathBuf::new(),
                title,
                description: frontmatter.description.clone(),
                frontmatter,
                body: body.to_string(),
            })
        })
        .collect()
}

fn assign_page_outputs(pages: &mut [Page]) -> Result<()> {
    let mut used = BTreeSet::new();
    let has_root_index = pages.iter().any(|page| {
        page.source_rel.components().count() == 1
            && page.source_rel.file_name().and_then(OsStr::to_str) == Some("index.md")
    });
    for page in pages {
        let candidate = output_path_for_markdown(&page.source_rel, has_root_index);
        page.output_rel = uniquify_path(candidate, &mut used);
    }
    Ok(())
}

fn output_path_for_markdown(relative: &Path, has_root_index: bool) -> PathBuf {
    if relative.components().count() == 1
        && relative.file_name().and_then(OsStr::to_str) == Some("index.md")
    {
        return PathBuf::from("index.html");
    }
    let mut output = sanitize_relative_path(relative);
    output.set_extension("html");
    if !has_root_index
        && relative.components().count() == 1
        && relative.file_name().and_then(OsStr::to_str) == Some("README.md")
    {
        output = PathBuf::from("readme.html");
    }
    output
}

fn discover_linked_assets(root: &Path, pages: &[Page], max_asset_size: u64) -> Result<Vec<Asset>> {
    let mut assets = BTreeMap::<PathBuf, PathBuf>::new();
    let page_sources = pages
        .iter()
        .map(|page| page.source.clone())
        .collect::<BTreeSet<_>>();
    for page in pages {
        for target in markdown_link_targets(&page.body) {
            let Some((path_part, _suffix)) = split_link_target(&target) else {
                continue;
            };
            let resolved = resolve_local_link(&page.source, &path_part);
            let Ok(source) = resolved else {
                continue;
            };
            if !is_under(&source, root) || !source.is_file() || page_sources.contains(&source) {
                continue;
            }
            if max_asset_size > 0
                && source.metadata().map(|meta| meta.len()).unwrap_or(0) > max_asset_size
            {
                eprintln!(
                    "  ! skipped linked asset over size limit: {}",
                    source.display()
                );
                continue;
            }
            let output_rel = sanitize_relative_path(source.strip_prefix(root).unwrap_or(&source));
            assets.entry(source).or_insert(output_rel);
        }
    }
    Ok(assets
        .into_iter()
        .map(|(source, output_rel)| Asset { source, output_rel })
        .collect())
}

fn write_page(
    output_dir: &Path,
    page: &Page,
    link_maps: &LinkMaps,
    site_title: &str,
    site_description: &Option<String>,
    site_properties: &BTreeMap<String, Value>,
    nav_html: &str,
) -> Result<()> {
    let body_html = render_markdown(page, link_maps);
    let is_home = page.output_rel == Path::new("index.html");
    let description = if is_home {
        site_description.as_deref().or(page.description.as_deref())
    } else {
        page.description.as_deref()
    };
    let webby_metadata = if is_home {
        render_webby_metadata(site_title, site_description.as_deref(), site_properties)?
    } else {
        String::new()
    };
    write_html_page(
        output_dir,
        &page.output_rel,
        PageRender {
            site_title,
            page_title: &page.title,
            source_path: Some(&page.source_rel.to_string_lossy()),
            description,
            meta_html: &render_meta(&page.frontmatter),
            body_html: &body_html,
            nav_html,
            root_href: &relative_href(&page.output_rel, Path::new("index.html"), ""),
            webby_metadata: &webby_metadata,
            is_home,
        },
    )?;

    Ok(())
}

struct PageRender<'a> {
    site_title: &'a str,
    page_title: &'a str,
    source_path: Option<&'a str>,
    description: Option<&'a str>,
    meta_html: &'a str,
    body_html: &'a str,
    nav_html: &'a str,
    root_href: &'a str,
    webby_metadata: &'a str,
    is_home: bool,
}

fn write_html_page(output_dir: &Path, output_rel: &Path, render: PageRender<'_>) -> Result<()> {
    let html = docs_env()
        .get_template("docs_page.html")
        .expect("docs_page.html template")
        .render(context! {
            site_title => render.site_title,
            page_title => render.page_title,
            source_path => render.source_path.unwrap_or(""),
            description => render.description.unwrap_or(""),
            meta_html => render.meta_html,
            body_html => render.body_html,
            nav_html => render.nav_html,
            root_href => render.root_href,
            webby_metadata => render.webby_metadata,
            is_home => render.is_home,
            css => include_str!("../templates/docs.css"),
        })
        .expect("render docs page");
    let target = output_dir.join(output_rel);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(target, html).map_err(Into::into)
}

fn write_synthetic_home(
    output_dir: &Path,
    site_title: &str,
    site_description: Option<&str>,
    properties: &BTreeMap<String, Value>,
    pages: &[Page],
) -> Result<()> {
    let mut body = String::from("<p>Generated from Markdown files.</p>\n<ul>\n");
    for page in pages {
        body.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>\n",
            html_escape(&page.output_rel.to_string_lossy()),
            html_escape(&page.title)
        ));
    }
    body.push_str("</ul>\n");
    let nav_html = render_nav(pages, Path::new("index.html"));
    let metadata = render_webby_metadata(site_title, site_description, properties)?;
    write_html_page(
        output_dir,
        Path::new("index.html"),
        PageRender {
            site_title,
            page_title: site_title,
            source_path: None,
            description: site_description,
            meta_html: "",
            body_html: &body,
            nav_html: &nav_html,
            root_href: "./index.html",
            webby_metadata: &metadata,
            is_home: true,
        },
    )
}

fn render_markdown(page: &Page, maps: &LinkMaps) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let events = Parser::new_ext(&page.body, options).map(|event| rewrite_event(event, page, maps));
    let mut rendered = String::new();
    html::push_html(&mut rendered, events);
    rendered
}

fn rewrite_event<'a>(event: Event<'a>, page: &Page, maps: &LinkMaps) -> Event<'a> {
    match event {
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Link {
            link_type,
            dest_url: rewrite_link(&page.source, &page.output_rel, dest_url, maps),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) => Event::Start(Tag::Image {
            link_type,
            dest_url: rewrite_link(&page.source, &page.output_rel, dest_url, maps),
            title,
            id,
        }),
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        other => other,
    }
}

fn rewrite_link<'a>(
    source: &Path,
    output_rel: &Path,
    dest_url: CowStr<'a>,
    maps: &LinkMaps,
) -> CowStr<'a> {
    let target = dest_url.to_string();
    let Some((path_part, suffix)) = split_link_target(&target) else {
        return dest_url;
    };
    let Ok(resolved) = resolve_local_link(source, &path_part) else {
        return dest_url;
    };
    if let Some(target_rel) = maps
        .pages
        .get(&resolved)
        .or_else(|| maps.assets.get(&resolved))
    {
        return CowStr::Boxed(relative_href(output_rel, target_rel, &suffix).into_boxed_str());
    }
    dest_url
}

fn markdown_link_targets(markdown: &str) -> Vec<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    Parser::new_ext(markdown, options)
        .filter_map(|event| match event {
            Event::Start(Tag::Link { dest_url, .. })
            | Event::Start(Tag::Image { dest_url, .. }) => Some(dest_url.to_string()),
            _ => None,
        })
        .collect()
}

fn render_nav(pages: &[Page], current_rel: &Path) -> String {
    let mut html = String::from("<ul class=\"nav-list\">\n");
    if !pages
        .iter()
        .any(|page| page.output_rel == Path::new("index.html"))
    {
        html.push_str("<li><a class=\"nav-link\" href=\"index.html\">Home</a></li>\n");
    }
    for page in pages {
        let href = relative_href(current_rel, &page.output_rel, "");
        let current = if page.output_rel == current_rel {
            " aria-current=\"page\""
        } else {
            ""
        };
        html.push_str(&format!(
            "<li><a class=\"nav-link\" href=\"{}\"{}>{}</a></li>\n",
            html_escape(&href),
            current,
            html_escape(&page.title)
        ));
    }
    html.push_str("</ul>\n");
    html
}

fn render_meta(frontmatter: &Frontmatter) -> String {
    let mut items = Vec::new();
    if let Some(kind) = &frontmatter.kind {
        items.push(format!("<span class=\"pill\">{}</span>", html_escape(kind)));
    }
    for tag in &frontmatter.tags {
        items.push(format!("<span class=\"pill\">#{}</span>", html_escape(tag)));
    }
    if let Some(timestamp) = &frontmatter.timestamp {
        items.push(format!(
            "<span class=\"pill\">{}</span>",
            html_escape(timestamp)
        ));
    }
    if let Some(resource) = &frontmatter.resource {
        items.push(format!(
            "<a class=\"pill\" href=\"{}\">resource</a>",
            html_escape(resource)
        ));
    }
    items.join("")
}

fn render_webby_metadata(
    title: &str,
    description: Option<&str>,
    properties: &BTreeMap<String, Value>,
) -> Result<String> {
    let mut object = Map::new();
    object.insert("title".to_string(), Value::String(title.to_string()));
    if let Some(description) = description {
        object.insert(
            "description".to_string(),
            Value::String(description.to_string()),
        );
    }
    object.insert(
        "properties".to_string(),
        Value::Object(
            properties
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
        ),
    );
    let json = serde_json::to_string_pretty(&Value::Object(object))
        .map_err(|e| err(format!("failed to serialize docs metadata: {e}")))?;
    Ok(format!(
        "<script type=\"application/webby+json\">\n{json}\n</script>"
    ))
}

fn copy_asset(asset: &Asset, output_dir: &Path) -> Result<()> {
    let target = output_dir.join(&asset.output_rel);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&asset.source, target)?;
    Ok(())
}

fn split_frontmatter(text: &str) -> (Frontmatter, &str) {
    let Some(rest) = text.strip_prefix("---\n") else {
        return (Frontmatter::default(), text);
    };
    let Some(end) = rest.find("\n---") else {
        return (Frontmatter::default(), text);
    };
    let yaml = &rest[..end];
    let after_marker = &rest[end + "\n---".len()..];
    let body = after_marker.strip_prefix('\n').unwrap_or(after_marker);
    let frontmatter = serde_yml::from_str::<Frontmatter>(yaml).unwrap_or_default();
    (frontmatter, body)
}

fn page_title(relative: &Path, frontmatter: &Frontmatter, body: &str) -> String {
    frontmatter
        .title
        .clone()
        .or_else(|| frontmatter.name.as_ref().map(|name| title_from_text(name)))
        .or_else(|| first_heading(body))
        .unwrap_or_else(|| title_from_path(relative))
}

fn first_heading(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        let trimmed = line.trim_start();
        trimmed
            .strip_prefix("# ")
            .map(|heading| heading.trim().trim_matches('#').trim().to_string())
            .filter(|heading| !heading.is_empty())
    })
}

fn split_link_target(target: &str) -> Option<(String, String)> {
    if target.is_empty()
        || target.starts_with('#')
        || target.starts_with("//")
        || target.contains("://")
        || target.starts_with("mailto:")
    {
        return None;
    }
    let split_at = target
        .char_indices()
        .find(|(_, ch)| *ch == '#' || *ch == '?')
        .map(|(idx, _)| idx)
        .unwrap_or(target.len());
    let path = percent_decode(&target[..split_at]);
    if path.is_empty() {
        return None;
    }
    Some((path, target[split_at..].to_string()))
}

fn resolve_local_link(source: &Path, target: &str) -> Result<PathBuf> {
    if target.starts_with('/') {
        return Err(err("absolute links are not local files"));
    }
    let path = source
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(target);
    fs::canonicalize(path).map_err(Into::into)
}

fn relative_href(from_rel: &Path, to_rel: &Path, suffix: &str) -> String {
    let from_parent = from_rel.parent().unwrap_or_else(|| Path::new(""));
    let from_parts = from_parent
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(_) => Some(".."),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut parts = from_parts
        .iter()
        .map(|part| (*part).to_string())
        .collect::<Vec<_>>();
    parts.extend(to_rel.components().filter_map(|component| match component {
        std::path::Component::Normal(part) => Some(part.to_string_lossy().to_string()),
        _ => None,
    }));
    let href = if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    };
    format!("{href}{suffix}")
}

fn sanitize_relative_path(relative: &Path) -> PathBuf {
    let parts = relative
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => {
                let text = part.to_string_lossy();
                Some(if text.starts_with('.') {
                    format!("_{}", text.trim_start_matches('.'))
                } else {
                    text.to_string()
                })
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    parts.iter().collect()
}

fn uniquify_path(candidate: PathBuf, used: &mut BTreeSet<String>) -> PathBuf {
    let mut path = candidate;
    let original = path.clone();
    let mut counter = 2;
    while used.contains(&path.to_string_lossy().to_ascii_lowercase()) {
        let stem = original
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("page");
        let extension = original.extension().and_then(OsStr::to_str).unwrap_or("");
        let name = if extension.is_empty() {
            format!("{stem}-{counter}")
        } else {
            format!("{stem}-{counter}.{extension}")
        };
        path = original.with_file_name(name);
        counter += 1;
    }
    used.insert(path.to_string_lossy().to_ascii_lowercase());
    path
}

fn should_skip_name(name: &OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(
            ".git"
                | ".hg"
                | ".svn"
                | ".wrangler"
                | ".DS_Store"
                | ".docme"
                | "node_modules"
                | "__pycache__"
                | "venv"
                | ".venv"
                | "logs"
                | "target"
        )
    )
}

fn validate_app_name(name: &str) -> Result<()> {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(err(format!("invalid docs app name '{name}'")));
    }
    Ok(())
}

fn is_under(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root).is_ok()
}

fn root_index(pages: &[Page]) -> Option<&Page> {
    pages
        .iter()
        .find(|page| page.output_rel == Path::new("index.html"))
}

fn title_slug(text: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn title_from_path(path: &Path) -> String {
    let stem = path.file_stem().and_then(OsStr::to_str).unwrap_or("Docs");
    title_from_text(stem)
}

fn title_from_text(text: &str) -> String {
    let stripped = text.trim();
    if stripped.is_empty() {
        return "Docs".to_string();
    }
    if stripped.chars().any(|ch| ch.is_ascii_lowercase()) {
        stripped
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|word| !word.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        stripped.to_string()
    }
}

fn percent_decode(value: &str) -> String {
    let mut output = Vec::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                output.push(hex);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).to_string()
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn docs_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template(
        "docs_page.html",
        include_str!("../templates/docs_page.html"),
    )
    .expect("docs page template");
    env
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_and_title() {
        let (frontmatter, body) = split_frontmatter(
            "---\ntype: Runbook\ntitle: Deploy Guide\ntags: [ops, release]\n---\n# Fallback\n",
        );

        assert_eq!(frontmatter.kind.as_deref(), Some("Runbook"));
        assert_eq!(frontmatter.title.as_deref(), Some("Deploy Guide"));
        assert_eq!(frontmatter.tags, vec!["ops", "release"]);
        assert_eq!(body, "# Fallback\n");
        assert_eq!(
            page_title(Path::new("deploy.md"), &frontmatter, body),
            "Deploy Guide"
        );
    }

    #[test]
    fn rewrites_markdown_links_to_relative_html() {
        let root =
            std::env::temp_dir().join(format!("webby-docs-link-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("a")).unwrap();
        fs::create_dir_all(root.join("b")).unwrap();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(root.join("a").join("one.md"), "").unwrap();
        fs::write(root.join("b").join("two.md"), "").unwrap();
        fs::write(root.join("assets").join("pic.png"), "").unwrap();
        let source = fs::canonicalize(root.join("a").join("one.md")).unwrap();
        let page = Page {
            source: source.clone(),
            source_rel: PathBuf::from("a/one.md"),
            output_rel: PathBuf::from("a/one.html"),
            title: "One".to_string(),
            description: None,
            frontmatter: Frontmatter::default(),
            body: "[Two](../b/two.md#part) ![Image](../assets/pic.png)".to_string(),
        };
        let maps = LinkMaps {
            pages: HashMap::from([(
                fs::canonicalize(root.join("b").join("two.md")).unwrap(),
                PathBuf::from("b/two.html"),
            )]),
            assets: HashMap::from([(
                fs::canonicalize(root.join("assets").join("pic.png")).unwrap(),
                PathBuf::from("assets/pic.png"),
            )]),
        };

        let html = render_markdown(&page, &maps);

        assert!(html.contains("href=\"../b/two.html#part\""));
        assert!(html.contains("src=\"../assets/pic.png\""));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn raw_html_is_escaped() {
        let page = Page {
            source: PathBuf::from("/tmp/docs/page.md"),
            source_rel: PathBuf::from("page.md"),
            output_rel: PathBuf::from("page.html"),
            title: "Page".to_string(),
            description: None,
            frontmatter: Frontmatter::default(),
            body: "<script>alert(1)</script>".to_string(),
        };

        let html = render_markdown(
            &page,
            &LinkMaps {
                pages: HashMap::new(),
                assets: HashMap::new(),
            },
        );

        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn percent_encoded_traversal_stays_unresolved_without_map() {
        assert_eq!(
            split_link_target("..%2Fsecret.md").unwrap().0,
            "../secret.md"
        );
    }
}
