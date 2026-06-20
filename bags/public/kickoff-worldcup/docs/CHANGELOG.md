# Changelog

## 2026-06-18

### Results Timeline App

Added a standalone static `index.html` that fetches FIFA World Cup 2026 match data, filters completed group-stage results, and renders each group as a horizontal date timeline with match result cards and team flags.

Added a `Justfile` with repeatable Webby commands for local and internal publishing.

Adjusted timeline placement to use match time on the horizontal axis and stack close same-day cards vertically within a group lane, preventing overlapping result cards.

### Future Fixtures

Modified the timeline to include upcoming group-stage fixtures and live matches from the FIFA feed, not just completed results.

Updated the header summary to report result count, upcoming fixture count, and completed-match goals.

Added fixture/live card styling and kept match-number/date labels from wrapping in compact fixture cards.

### Compressed Spacing

Changed timeline layout from proportional elapsed-time spacing to compact date columns, reducing empty horizontal gaps between group matchdays.

Stacked same-date cards vertically and shortened visible match identifiers to fit denser fixture cards.

### Standings Tables

Added selected-group standings tables below match cards for dates with completed group matches.

Filtered standings tables so future-only fixture dates do not show a carried-forward table.

### Filter Controls

Changed group chips from single-select to additive OR filters, with `All` clearing active group selections.

Added a `Show tables` checkbox. It defaults off in All, turns on when the first group is selected, and can also be enabled manually in All to show standings tables for every group.

### Today Marker

Added a yellow current-day marker and `Today` badge when the browser's current date appears in the rendered timeline.

Added default horizontal auto-scroll to bring the current date column into view after rendering.

### Timezone Conversion

Changed match date/time display from FIFA `LocalDate` values to conversion from FIFA's UTC `Date` field using the selected browser timezone.

Added a timezone selector that defaults to the browser timezone and supports the browser's IANA timezone list when available.

Updated date columns, match labels, standings table dates, and the current-day marker to recompute from the selected timezone.

Replaced the IANA timezone dropdown with a compact offset dropdown sorted from UTC-12:00 to UTC+12:00, using one representative city/country label per offset.

### Compact Controls

Replaced verbose group chips with a compact segmented `All | A | B | ... | L` selector.

Changed `All` into a virtual all-selected button that stays highlighted whenever every group is selected.

Moved timezone and table controls into the same top row as the group selector.

Moved summary stats from the header to a bottom strip below the timeline, adding live count and date range there.

### Kickoff Branding

Renamed the page and header to `Kickoff`, removed the header subheading, and added a football image as the favicon and header logo.

Changed the page background to a pitch-green treatment and added icon attribution in the source footer.

### Single-Shell Timeline Layout

Moved the app into one full-viewport timeline shell with the compact Kickoff logo, filters, timezone picker, and table toggle in the top row.

Removed the outer page header/background treatment and made the timeline body the scrollable viewport, with stats and source attribution pinned inside the same app shell.

### Group Filter Semantics

Changed `All` from a select-everything state into a virtual unfiltered state. Group buttons now start unselected by default, clicking a group from `All` filters to that group, and selecting every group normalizes back to `All`.

Added a compact `Group` label beside the segmented group selector.

### Timeline Polish

Changed match-card times to AM/PM format and shortened them to time-only labels within each date column.

Collapsed the large summary strip into a compact footer line with source attribution.

Updated the `Kickoff` wordmark color to `#16841d` and aligned lane gridlines with the date-axis separators by sharing the active column width.

### Stable Table Toggle

Set the no-table timeline column width to match the table-enabled width so date columns and match positions do not shift when `Show tables` is toggled.

Preserved the timeline viewport scroll offset during table-toggle renders.

### Persisted View State

Added `localStorage` persistence for group selection, table visibility, timezone offset, and timeline scroll position across page refreshes.

Restored saved scroll after match data renders and normalized legacy all-groups selections to the virtual `All` filter state.

### 3D Ball Logo Texture

Replaced the custom procedural soccer-ball patches with a real THREEx SportBalls football texture mapped onto the Three.js logo sphere.

Added the texture source attribution and vendored the corresponding MIT license file with the asset.

### Grass Timeline Columns

Added alternating green date-column backgrounds and subtle grass-grain texture layers to the timeline lanes.

Kept the textured column backgrounds aligned to the same date-column width used by the axis separators.

### Grass Image Columns

Replaced the synthetic grass-grain layers with the provided `grass1.jpg` and `grass2.jpg` textures.

Used a masked pseudo-layer to show `grass2.jpg` only on alternating date columns while keeping separator lines aligned.

### Single Grass Texture Alternation

Changed the lane background to use only `grass2.jpg`, with a masked dark overlay on alternating date columns.

### Project Rename

Updated the Webby app slug to `kickoff-worldcup` for the project rename.

### Public Publish

Renamed the table visibility toggle from `Show tables` to `Standings`.

Published the app to the public Webby bag and generated the public preview card.

### Today Jump Control

Moved `grass2.jpg` into `assets/` and updated the timeline lane background reference.

Added a compact `TODAY` jump button that points left or right when the current-day line is outside the comfortable viewport area, and shows a dot when today is already visible.

Increased date-axis label text size.

### Webby Cleanup

Removed old internal/local Webby entries for the pre-rename app and removed the duplicate internal `kickoff-worldcup` app copy, leaving the public Webby entry as canonical.

### README

Added a minimal README with the Webby preview image and a one-line project description.
