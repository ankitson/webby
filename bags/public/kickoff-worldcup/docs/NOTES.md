# Notes

## 2026-06-18 - World Cup Results Timeline

Goal: create a static Webby-hosted visualization of completed FIFA World Cup 2026 group-stage results.

Decisions:
- Use FIFA's calendar API directly from the browser: `https://api.fifa.com/api/v3/calendar/matches?language=en&count=104&idCompetition=17&idSeason=285023`.
- Filter to `MatchStatus === 0`, which FIFA's frontend bundle maps to played/completed matches.
- Show only group-stage matches by grouping on `GroupName`, with each group as a lane and local match date as the horizontal axis.

Next steps:
- Publish internally with `webby add . --name worldcup-2026-results-timeline -b internal`.

Verification:
- Internal Webby URL returned HTTP 200.
- Browser render loaded 26 completed group-stage matches across all 12 groups.
- Group filter interaction was checked with Group A, which showed 3 completed results.
- Desktop and mobile screenshots were inspected after adding vertical stacking for close same-day matches.

## 2026-06-18 - Future Group Fixtures

Goal: extend the timeline to show upcoming group-stage fixtures as well as completed results.

Decisions:
- Include FIFA match statuses `0` played, `1` upcoming, and `3` live.
- Keep completed results and upcoming fixtures in the same group lanes so each group reads as a full group-stage path.
- Style upcoming fixtures with dashed cards and a `Fixture` badge; live matches receive a `Live` badge.

Verification:
- Browser render loaded 72 group-stage matches: 26 results, 45 upcoming fixtures, and 1 live match.
- Group A filter showed 6 total group matches: 3 results and 3 upcoming fixtures.
- Desktop and mobile screenshots were inspected after the fixture card styling update.

## 2026-06-18 - Compressed Timeline Spacing

Goal: reduce empty horizontal space in group timelines.

Decisions:
- Replace proportional elapsed-time spacing with equal-width date columns.
- Stack multiple same-date matches vertically inside each group lane.
- Reduce the minimum timeline width so filtered group views can fit in one viewport.
- Shorten visible match labels from `Match 53` to `M53` to keep compact fixture cards readable.

Verification:
- Group A filtered view showed all six group matches across three compact date columns in one desktop viewport.
- Browser smoke check confirmed the expanded data still loaded with 45 upcoming fixtures.

## 2026-06-18 - Hide Future-Day Tables

Goal: keep selected-group standings tables off future-only fixture dates.

Decision:
- Render standings tables only for date columns where the selected group has at least one completed match.
- Keep future fixture cards visible without carrying the current table forward.

Verification:
- Group A still showed Jun 24 fixture cards.
- Group A standings titles were limited to `After Jun 11` and `After Jun 18`; `After Jun 24` was not rendered.

## 2026-06-18 - Multi-Group Filters and Table Toggle

Goal: let group chips behave as OR filters and make standings tables independently toggleable.

Decisions:
- `All` now clears group filters and resets `Show tables` off.
- Group chips are independent toggles, so multiple groups can be selected at once.
- Selecting the first group from the All view defaults `Show tables` on.
- The `Show tables` checkbox can be enabled in the All view to render standings tables across all groups.

Verification:
- Initial All view had `Show tables` unchecked and no standings cards.
- Selecting Group A checked `Show tables` and rendered Group A tables.
- Selecting Group B while Group A was active showed both group rows.
- Returning to All reset `Show tables` off.
- Checking `Show tables` in All rendered tables across all group rows.

## 2026-06-18 - Today Marker and Default Scroll

Goal: make the current tournament day immediately visible.

Decisions:
- Add a yellow vertical marker at the start of the current date column when that date is present in the rendered timeline.
- Add a `Today` badge in the date axis for the current date.
- After each render, horizontally scroll the timeline so the current date column is visible by default.

Verification:
- Browser date resolved to June 18, 2026.
- The rendered timeline included a `today-line` and visible `TODAY` badge on Jun 18.
- The served page loaded with the horizontal scroll offset near the Jun 18 column.

## 2026-06-18 - Viewer Timezone Conversion

Goal: show match dates and times in the viewer's selected timezone instead of FIFA venue-local time.

Decisions:
- Use FIFA's UTC `Date` field as the canonical match timestamp.
- Initialize the timezone selector from the browser timezone.
- Populate the selector with common North American/World Cup zones and the browser's supported IANA timezone list when available.
- Recompute match date columns, match card times, standings table dates, and the today marker from the selected timezone.

Verification:
- Browser default timezone resolved to `America/Vancouver`.
- Group A match 1 displayed as `Jun 11 12:00` in `America/Vancouver`.
- Switching to `Asia/Tokyo` changed Group A match 1 to `Jun 12 04:00`.
- The timezone selector included `Asia/Tokyo` and 419 total browser-supported options.
- The today marker followed the selected timezone.

## 2026-06-18 - Offset Timezone Selector

Goal: make the timezone selector concise and ordered by actual UTC offset.

Decisions:
- Replace the full IANA timezone list with one representative city/country label per UTC offset.
- Sort options from UTC-12:00 through UTC+12:00.
- Keep conversion as fixed UTC offsets so each option is unambiguous.

Verification:
- Browser check showed 34 unique offset options.
- First option was `UTC-12:00 - Baker Island, USA`.
- Last option was `UTC+12:00 - Auckland, New Zealand`.
- Options were sorted by numeric offset and unique.
- Browser default selected `UTC-07:00 - Vancouver, Canada (browser)`.
- Switching to `UTC+09:00 - Tokyo, Japan` changed Group A match 1 to `Jun 12 04:00`.

## 2026-06-18 - Compact Top Controls and Bottom Summary

Goal: make the filter controls denser and move dashboard stats out of the header.

Decisions:
- Replace large `Group A` style chips with a compact segmented selector: `All | A | B | ... | L`.
- Treat `All` as a virtual all-selected state. Clicking it selects every group, and it is highlighted whenever all individual group buttons are selected.
- Keep timezone and `Show tables` controls on the same top control row as the group selector.
- Move results, fixtures, goals, live count, and date range into a bottom summary strip below the timeline.

Verification:
- Browser check showed group button labels as `All, A, B, C, ... L`.
- Initial state had All highlighted and all 12 group buttons selected.
- Toggling Group A off cleared All; toggling Group A back on restored All.
- Header no longer contained summary stats.
- Bottom summary labels rendered in order: results, fixtures, goals, live, date range.

## 2026-06-18 - Kickoff Branding

Goal: rebrand the app as `Kickoff` with a football identity.

Decisions:
- Rename the document title and visible header to `Kickoff`.
- Remove the header subheading so the top bar stays compact.
- Use the Flaticon soccer ball PNG as both favicon and header logo.
- Switch the page backdrop to a football-pitch green with subtle field-style striping.
- Add a footer attribution link for the icon source.

Verification:
- Pending browser smoke check after Webby restage.

## 2026-06-18 - Single-Shell Timeline Layout

Goal: remove the page wrapper treatment and make the timeline the whole app surface.

Decisions:
- Move the `Kickoff` logo, group selector, timezone picker, and table toggle into one compact top row inside the timeline shell.
- Remove the separate page header and pitch-green outer background.
- Make the timeline viewport the horizontal scroll container so the top and bottom bars stay fixed in the app shell.
- Keep summary stats and attribution as bottom rows inside the same full-screen shell.

Verification:
- Browser check confirmed there is no body-level header, the shell starts at the viewport origin, the logo image loads, controls live in the toolbar, and the timeline viewport owns horizontal scrolling.
- Screenshot inspected at 1280px wide with the current-day column visible.

## 2026-06-18 - Group Filter Semantics

Goal: make the group selector behave like standard filter chips.

Decisions:
- Treat `All` as an unfiltered virtual state instead of selecting every group chip.
- Keep only `All` highlighted by default.
- Clicking a group from `All` now selects that group alone.
- Selecting additional group chips keeps additive OR filtering.
- Clearing the last selected group, clicking `All`, or selecting every group manually normalizes back to `All`.
- Add a compact `Group` label before the selector.

Verification:
- Browser check confirmed default `All` only, `A` selects only Group A, `A+B` selects both, clearing the last selected group returns to `All`, and selecting every group manually normalizes to `All`.

## 2026-06-18 - Compact Footer and Timeline Polish

Goal: tighten the chrome and improve timeline readability.

Decisions:
- Show match-card times in AM/PM format.
- Use time-only labels on match cards because the date is already represented by the day column.
- Collapse the summary stats into a compact footer line with the source links.
- Style the `Kickoff` wordmark in `#16841d` with a slightly larger, heavier logo treatment.
- Drive lane gridlines from the same active date-column width used by the axis ticks.

Verification:
- Browser check confirmed AM/PM time labels, a 30px compact footer, `rgb(22, 132, 29)` wordmark color, and aligned lane/tick separator positions.

## 2026-06-18 - Stable Column Width on Table Toggle

Goal: prevent the timeline columns from shifting when standings tables are toggled.

Decisions:
- Use the table-friendly 292px date-column width for both table and non-table modes.
- Keep lane gridline fallbacks at the same 292px width.
- Preserve the timeline viewport scroll offset when the `Show tables` checkbox re-renders the timeline.

Verification:
- Browser toggle check confirmed column width, timeline width, date tick positions, match card x positions, and horizontal scroll offset stayed unchanged when turning tables off and back on.

## 2026-06-18 - Persisted View State

Goal: preserve the user's view after refreshing the page.

Decisions:
- Store group filter selection, table visibility, timezone offset, and timeline scroll position in `localStorage`.
- Restore saved state after FIFA match data loads so group filters can be validated against available groups.
- Keep first-time visits on the existing current-day auto-scroll behavior.
- Preserve table-toggle scroll behavior while also writing the updated view state.
- Normalize older saved states with every group selected back to the virtual `All` state.

Verification:
- Browser reload check restored Groups A+B, tables off, UTC+09:00 Tokyo, visible Group A/B rows, and the saved horizontal scroll offset after the data rendered.

## 2026-06-18 - Textured 3D Soccer Ball Logo

Goal: replace the hand-built soccer pattern with a real texture-mapped 3D ball.

Decisions:
- Vendor Three.js locally for the small header canvas.
- Use the THREEx SportBalls football texture from `images/Footballballfree...jpg`.
- Use the same texture as a subtle bump map on the sphere.
- Keep the existing slow 3D rotation and source attribution.
- Vendor the THREEx SportBalls MIT license alongside the texture asset.

Verification:
- Browser WebGL check confirmed nonblank canvas pixels, black and white texture regions, and changing frame checksums while rotating.
- Screenshot inspected at actual header size.

## 2026-06-18 - Grass Column Backgrounds

Goal: make the timeline columns feel more like a football pitch.

Decisions:
- Add alternating light/dark green backgrounds per date column.
- Add two subtle repeated grain layers to suggest grass texture.
- Keep separator lines tied to the same date-column width so they remain aligned with the date axis.

Verification:
- Browser check confirmed the lane background uses a 292px separator layer and 584px alternating stripe layer.
- Lane separator position still aligned with the date header tick.
- Screenshot inspected for readability behind match cards.

## 2026-06-18 - Grass Image Columns

Goal: use the provided grass textures instead of synthetic grass gradients.

Decisions:
- Use `grass1.jpg` as the base lane background.
- Reveal `grass2.jpg` only on every other date column through a repeated column-width mask.
- Keep separator lines as a separate pseudo-layer so they still align with the date axis.
- Add a light wash over both grass images for card readability, with the alternate columns slightly darker/saturated.

Verification:
- Browser style check confirmed `grass1.jpg` on the lane, `grass2.jpg` on the masked pseudo-layer, and a 584px mask period for two 292px columns.
- Screenshot confirmed visible alternation and readable match cards.

## 2026-06-18 - Single Grass Texture Alternation

Goal: remove `grass1.jpg` from the timeline lanes and use only `grass2.jpg`.

Decision:
- Use `grass2.jpg` as the lane background everywhere.
- Add a masked dark-green overlay to every other date column for alternation.

Verification:
- Browser style check confirmed no lane `grass1.jpg` reference, `grass2.jpg` as the base, and a 584px alternating overlay mask.
- Screenshot confirmed the columns alternate while keeping the same grass texture.

## 2026-06-18 - Project Rename

Goal: rename the project to `kickoff-worldcup`.

Decisions:
- Update the Webby app slug in the `Justfile` to `kickoff-worldcup`.
- Rename the workspace directory from `worldcup-2026-results-timeline` to `kickoff-worldcup`.
- Commit and push the initial project tree to the configured `origin` remote.

Verification:
- New Webby URL loaded at `https://tools.home.ankitson.com/kickoff-worldcup/`.
- Browser smoke check confirmed the `Kickoff` page renders, Group A appears, and `grass2.jpg` is applied.

## 2026-06-18 - Public Webby Publish

Goal: move `kickoff-worldcup` to the public Webby bag.

Decisions:
- Rename the table toggle label to `Standings`.
- Publish the folder app to the public Webby bag with Webby.

Verification:
- Public Webby publish completed at `https://mini.ankitson.com/kickoff-worldcup/`.
- Browser smoke check confirmed the `Standings` toggle label and `Kickoff` page render on the public URL.
- Public preview card generated at `/projects/webby/public/.webby-previews/kickoff-worldcup.jpg`.
- Removed staged `.git` metadata from Webby public/internal bag copies and redeployed the public bag.

## 2026-06-18 - Webby Bag Cleanup

Goal: keep `kickoff-worldcup` only in the public Webby bag.

Decisions:
- Remove the old `worldcup-2026-results-timeline` app from internal and local bags.
- Remove the duplicate `kickoff-worldcup` app from the internal bag.
- Remove stale internal/local preview images for the old and duplicate app entries.

Verification:
- Webby listings show `kickoff-worldcup` only under the public bag.
- Preview files for the app remain only under `/projects/webby/public/.webby-previews/`.

## 2026-06-18 - README Preview

Goal: add a minimal repository README.

Decision:
- Copy the public Webby preview image into `assets/preview.jpg`.
- Add `README.md` with the preview image and a one-line project description.
