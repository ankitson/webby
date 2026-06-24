const TEMPLATE = document.createElement("template");

TEMPLATE.innerHTML = `
  <style>
    :host {
      color-scheme: light dark;
      --webby-paper: var(--webby-theme-paper, #f7f7f2);
      --webby-ink: var(--webby-theme-ink, #171713);
      --webby-muted: var(--webby-theme-muted, #6c6a60);
      --webby-line: var(--webby-theme-line, rgba(23,23,19,.14));
      --webby-tile: var(--webby-theme-tile, #ecebe3);
      --webby-accent: var(--webby-theme-accent, #d25f3b);
      --webby-gap: var(--webby-theme-gap, clamp(14px, 2vw, 22px));
      --webby-column-gap: clamp(10px, 1.8vw, 18px);
      --webby-card-radius: 8px;
      --webby-card-min-width: 280px;
      --webby-card-stable-width: 292px;
      --webby-card-max-width: 800px;
      --webby-card-max-height: 800px;
      --webby-card-track-size: minmax(min(100%, var(--webby-card-min-width)), 1fr);
      --webby-grid-justify-content: start;
      --webby-preview-filter: var(--webby-theme-preview-filter, none);
      --webby-preview-overlay: var(--webby-theme-preview-overlay, transparent);
      --webby-preview-inset-shadow: var(--webby-theme-preview-inset-shadow, 0 1px 0 rgba(255,255,255,.45) inset);
      --webby-card-title-color: var(--webby-theme-card-title-color, var(--webby-ink));
      --webby-card-description-color: var(--webby-theme-card-description-color, var(--webby-muted));
      --webby-font: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      display: block;
      color: var(--webby-ink);
      font-family: var(--webby-font);
    }

    :host([data-theme="dark"]) {
      --webby-paper: var(--webby-theme-paper, #11120f);
      --webby-ink: var(--webby-theme-ink, #f2f0e8);
      --webby-muted: var(--webby-theme-muted, #aba89c);
      --webby-line: var(--webby-theme-line, rgba(255,255,255,.16));
      --webby-tile: var(--webby-theme-tile, #1b1d18);
      --webby-accent: var(--webby-theme-accent, #ff8a62);
      --webby-preview-filter: var(--webby-theme-preview-filter, brightness(.72) contrast(.94) saturate(.88));
      --webby-preview-overlay: var(--webby-theme-preview-overlay, rgba(0,0,0,.18));
      --webby-preview-inset-shadow: var(--webby-theme-preview-inset-shadow, 0 1px 0 rgba(255,255,255,.08) inset);
      --webby-card-title-color: var(--webby-theme-card-title-color, #d8d3c7);
      --webby-card-description-color: var(--webby-theme-card-description-color, #8f8a7d);
    }

    * { box-sizing: border-box; }
    a { color: inherit; }

    .sections {
      display: grid;
      gap: clamp(18px, 3vw, 34px);
    }

    .category-title {
      margin: 0 0 10px;
      color: var(--webby-muted);
      font-size: 12px;
      font-weight: 760;
      letter-spacing: .09em;
      line-height: 1.2;
      text-transform: uppercase;
    }

    .grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, var(--webby-card-track-size));
      gap: var(--webby-gap) var(--webby-column-gap);
      justify-content: var(--webby-grid-justify-content);
      justify-items: stretch;
    }

    .site {
      width: 100%;
      min-width: 0;
      max-width: min(100%, var(--webby-card-max-width));
      max-height: var(--webby-card-max-height);
      contain: layout paint;
      isolation: isolate;
    }

    .preview-link {
      position: relative;
      display: block;
      aspect-ratio: 16 / 10;
      max-height: calc(var(--webby-card-max-height) - 42px);
      overflow: hidden;
      border: 1px solid var(--webby-line);
      border-radius: var(--webby-card-radius);
      background: var(--webby-tile);
      box-shadow: var(--webby-preview-inset-shadow);
      outline: none;
    }

    .preview-link::after {
      content: "";
      position: absolute;
      inset: 0;
      z-index: 1;
      pointer-events: none;
      background: var(--webby-preview-overlay);
    }

    .preview {
      position: absolute;
      inset: 0;
      z-index: 0;
      overflow: hidden;
      background-image: var(--preview-image, none);
      background-position: center top;
      background-size: cover;
      filter: var(--webby-preview-filter);
    }

    .site-caption {
      display: flex;
      align-items: baseline;
      gap: 7px;
      min-width: 0;
      margin-top: 7px;
    }

    .site-title {
      min-width: 0;
      overflow: hidden;
      color: var(--webby-card-title-color);
      text-decoration: none;
      text-overflow: ellipsis;
      white-space: nowrap;
      font-size: clamp(14px, 1.4vw, 17px);
      font-weight: 680;
      line-height: 1.25;
    }

    .site-description {
      margin: 3px 0 0;
      overflow: hidden;
      color: var(--webby-card-description-color);
      text-overflow: ellipsis;
      white-space: nowrap;
      font-size: 13px;
      line-height: 1.25;
    }

    .temp-label {
      flex: 0 0 auto;
      border: 1px solid var(--webby-line);
      border-radius: 999px;
      padding: 1px 6px 2px;
      color: var(--webby-muted);
      font-size: 10px;
      font-weight: 700;
      line-height: 1.2;
      text-transform: uppercase;
      letter-spacing: .04em;
    }

    .site:hover .preview-link,
    .site:focus-within .preview-link {
      border-color: var(--webby-accent);
    }

    .site-title:hover {
      color: var(--webby-accent);
    }

    .preview-link:focus-visible,
    .site-title:focus-visible {
      outline: 2px solid var(--webby-accent);
      outline-offset: 3px;
    }

    .empty {
      margin: 0;
      min-height: 50vh;
      display: grid;
      place-items: center;
      color: var(--webby-muted);
    }

    :host([variant="compact"]) .grid {
      grid-template-columns: repeat(auto-fit, minmax(min(100%, 220px), 1fr));
      gap: 10px;
    }

    :host([variant="compact"]) .site {
      border: 1px solid var(--webby-line);
      border-radius: var(--webby-card-radius);
      padding: 10px;
      background: var(--webby-tile);
    }

    :host([variant="compact"]) .preview-link {
      display: none;
    }

    :host([variant="compact"]) .site-caption {
      margin-top: 0;
    }

    @media (max-width: 560px) {
      .grid {
        grid-template-columns: 1fr;
        gap: 10px;
      }

      .preview-link {
        aspect-ratio: 4 / 3;
      }

      .site-title,
      .site-description {
        white-space: normal;
        overflow-wrap: anywhere;
      }
    }
  </style>
  <div class="root" part="root"></div>
`;

function previewSlug(value) {
  return String(value || "")
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function groupItems(items, property) {
  const groups = new Map();
  for (const item of items) {
    const group = property ? item.properties[property] : item.category;
    const name = group === undefined || group === null || group === "" ? "Sites" : String(group);
    if (!groups.has(name)) groups.set(name, []);
    groups.get(name).push(item);
  }
  return groups;
}

function normalizeItem(item) {
  const id = String(item.id || item.title || item.href || "card");
  const properties = isPlainObject(item.properties) ? { ...item.properties } : {};
  return {
    id,
    title: String(item.title || id),
    href: String(item.href || "#"),
    description: item.description ? String(item.description) : "",
    category: item.category ? String(item.category) : (properties.category ? String(properties.category) : ""),
    properties,
    tmp: Boolean(item.tmp),
    previewUrl: item.previewUrl ? String(item.previewUrl) : "",
  };
}

function readPixelValue(style, name, fallback) {
  const value = Number.parseFloat(style.getPropertyValue(name));
  return Number.isFinite(value) ? value : fallback;
}

export class WebbyCardGrid extends HTMLElement {
  static observedAttributes = [
    "group-by-category",
    "group-by-property",
    "preview-base",
    "show-descriptions",
    "stable-card-width",
    "variant",
  ];

  constructor() {
    super();
    this._items = [];
    this._layoutFrame = 0;
    this.attachShadow({ mode: "open" });
    this.shadowRoot.append(TEMPLATE.content.cloneNode(true));
    this._root = this.shadowRoot.querySelector(".root");
    this._resizeObserver = new ResizeObserver(() => this.scheduleGridLayout());
  }

  connectedCallback() {
    this._resizeObserver.observe(this);
    this.render();
  }

  disconnectedCallback() {
    this._resizeObserver.disconnect();
    if (this._layoutFrame) cancelAnimationFrame(this._layoutFrame);
  }

  attributeChangedCallback() {
    this.render();
  }

  set items(value) {
    this._items = Array.isArray(value) ? value.map(normalizeItem) : [];
    this.render();
  }

  get items() {
    return this._items;
  }

  get groupByCategory() {
    return this.hasAttribute("group-by-category");
  }

  get groupByProperty() {
    return this.getAttribute("group-by-property") || "";
  }

  get showDescriptions() {
    return this.hasAttribute("show-descriptions");
  }

  get stableCardWidth() {
    return this.hasAttribute("stable-card-width") && this.getAttribute("variant") !== "compact";
  }

  get previewBase() {
    return this.getAttribute("preview-base") || "./webby-previews/";
  }

  render() {
    if (!this._root) return;
    this._root.replaceChildren();

    if (!this._items.length) {
      const empty = document.createElement("p");
      empty.className = "empty";
      empty.textContent = "Nothing here yet.";
      this._root.append(empty);
      return;
    }

    if (this.groupByCategory || this.groupByProperty) {
      const sections = document.createElement("div");
      sections.className = "sections";
      sections.setAttribute("part", "sections");
      for (const [category, items] of groupItems(this._items, this.groupByProperty)) {
        sections.append(this.renderSection(category, items, true));
      }
      this._root.append(sections);
      this.scheduleGridLayout();
      return;
    }

    this._root.append(this.renderGrid(this._items));
    this.scheduleGridLayout();
  }

  renderSection(category, items, showTitle) {
    const section = document.createElement("section");
    section.className = "category";
    section.setAttribute("part", "category");
    if (showTitle) {
      const title = document.createElement("h2");
      title.className = "category-title";
      title.setAttribute("part", "category-title");
      title.textContent = category;
      section.append(title);
    }
    section.append(this.renderGrid(items));
    return section;
  }

  renderGrid(items) {
    const grid = document.createElement("div");
    grid.className = "grid";
    grid.setAttribute("part", "grid");
    grid.dataset.count = String(items.length);
    for (const item of items) {
      grid.append(this.renderCard(item));
    }
    return grid;
  }

  scheduleGridLayout() {
    if (this._layoutFrame) cancelAnimationFrame(this._layoutFrame);
    this._layoutFrame = requestAnimationFrame(() => {
      this._layoutFrame = 0;
      this.applyGridLayout();
    });
  }

  applyGridLayout() {
    for (const grid of this.shadowRoot.querySelectorAll(".grid")) {
      const count = Number.parseInt(grid.dataset.count || "0", 10);
      if (!count) continue;
      grid.style.gridTemplateColumns = "";
      grid.style.columnGap = "";
      grid.style.justifyContent = "";
      grid.style.maxWidth = "";

      if (this.stableCardWidth) {
        this.applyStableGridLayout(grid, count);
        continue;
      }

      const hostStyle = getComputedStyle(this);
      const gridStyle = getComputedStyle(grid);
      const cardMax = readPixelValue(hostStyle, "--webby-card-max-width", 800);
      const columnGap = Number.parseFloat(gridStyle.columnGap);
      if (!Number.isFinite(cardMax) || !Number.isFinite(columnGap)) continue;
      grid.style.maxWidth = `${(count * cardMax) + (Math.max(0, count - 1) * columnGap)}px`;
    }
  }

  applyStableGridLayout(grid, count) {
    const hostStyle = getComputedStyle(this);
    const gridStyle = getComputedStyle(grid);
    const availableWidth = grid.parentElement.getBoundingClientRect().width;
    if (!Number.isFinite(availableWidth) || availableWidth <= 0) return;

    const minWidth = readPixelValue(hostStyle, "--webby-card-min-width", 280);
    const targetWidth = readPixelValue(hostStyle, "--webby-card-stable-width", minWidth);
    const maxWidth = readPixelValue(hostStyle, "--webby-card-max-width", 800);
    const minGap = Number.parseFloat(gridStyle.columnGap);
    const columnGap = Number.isFinite(minGap) ? minGap : 0;
    const cappedTarget = Math.min(Math.max(targetWidth, minWidth), maxWidth);
    const possibleColumns = Math.max(
      1,
      Math.floor((availableWidth + columnGap) / (cappedTarget + columnGap)),
    );
    const columns = Math.min(count, possibleColumns);

    if (count <= possibleColumns) {
      const fillWidth = columns === 1
        ? availableWidth
        : (availableWidth - (columnGap * (columns - 1))) / columns;
      const cardWidth = Math.min(maxWidth, Math.max(minWidth, fillWidth));
      const contentWidth = (cardWidth * columns) + (columnGap * Math.max(0, columns - 1));
      grid.style.gridTemplateColumns = `repeat(${columns}, minmax(0, ${cardWidth}px))`;
      grid.style.maxWidth = `${Math.min(availableWidth, contentWidth)}px`;
      return;
    }

    const cardWidth = columns === 1
      ? Math.min(maxWidth, Math.max(minWidth, availableWidth))
      : cappedTarget;
    const columnSpace = availableWidth - (cardWidth * columns);
    const gap = columns > 1 ? Math.max(columnGap, columnSpace / (columns - 1)) : columnGap;

    grid.style.gridTemplateColumns = `repeat(${columns}, minmax(0, ${cardWidth}px))`;
    grid.style.columnGap = `${gap}px`;
  }

  renderCard(item) {
    const previewUrl = item.previewUrl || `${this.previewBase}${previewSlug(item.id)}.webp`;

    const card = document.createElement("article");
    card.className = "site";
    card.setAttribute("part", "card");
    card.style.setProperty("--preview-image", `url("${previewUrl}")`);

    const previewLink = document.createElement("a");
    previewLink.className = "preview-link";
    previewLink.setAttribute("part", "preview-link");
    previewLink.href = item.href;
    previewLink.setAttribute("aria-label", `Open ${item.title}`);

    const preview = document.createElement("div");
    preview.className = "preview";
    preview.setAttribute("part", "preview");
    preview.setAttribute("aria-hidden", "true");
    previewLink.append(preview);
    card.append(previewLink);

    const caption = document.createElement("div");
    caption.className = "site-caption";
    caption.setAttribute("part", "caption");

    const title = document.createElement("a");
    title.className = "site-title";
    title.setAttribute("part", "title");
    title.href = item.href;
    title.textContent = item.title;
    caption.append(title);

    if (item.tmp) {
      const label = document.createElement("span");
      label.className = "temp-label";
      label.setAttribute("part", "temp-label");
      label.textContent = "temp";
      caption.append(label);
    }

    card.append(caption);

    if (this.showDescriptions && item.description) {
      const description = document.createElement("p");
      description.className = "site-description";
      description.setAttribute("part", "description");
      description.textContent = item.description;
      card.append(description);
    }

    return card;
  }
}

if (!customElements.get("webby-card-grid")) {
  customElements.define("webby-card-grid", WebbyCardGrid);
}
