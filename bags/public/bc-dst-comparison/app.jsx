import { useState, useMemo } from "react";
import { createRoot } from "react-dom/client";
import { AreaChart, Area, XAxis, YAxis, Tooltip, ResponsiveContainer, ReferenceLine, CartesianGrid } from "recharts";

// Solar calculation for Vancouver (49.28°N, -123.12°W)
function getSunTimes(dayOfYear, lat = 49.28) {
  const rad = Math.PI / 180;
  const decl = 23.45 * Math.sin(rad * (360 / 365) * (dayOfYear - 81));
  const ha = Math.acos(
    -Math.tan(lat * rad) * Math.tan(decl * rad)
  ) / rad;
  const solarNoon = 12.0 + (123.12 / 15) - (-8); // approximate for UTC-8 base
  const sunrise = 12 - ha / 15;
  const sunset = 12 + ha / 15;
  return { sunrise, sunset };
}

function dayOfYear(month, day) {
  const daysInMonth = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
  let d = day;
  for (let i = 1; i < month; i++) d += daysInMonth[i];
  return d;
}

function formatTime(hours) {
  const h = Math.floor(hours);
  const m = Math.round((hours - h) * 60);
  const hh = h % 12 === 0 ? 12 : h % 12;
  const ampm = h < 12 ? "AM" : "PM";
  return `${hh}:${m.toString().padStart(2, "0")} ${ampm}`;
}

function formatHourAxis(val) {
  const h = Math.floor(val);
  const hh = h % 12 === 0 ? 12 : h % 12;
  const ampm = h < 12 ? "a" : "p";
  return `${hh}${ampm}`;
}

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

// DST transitions (2nd Sunday of March, 1st Sunday of November)
const SPRING_FORWARD = dayOfYear(3, 8); // approx
const FALL_BACK = dayOfYear(11, 1); // approx

function generateData() {
  const data = [];
  for (let doy = 1; doy <= 365; doy++) {
    const sun = getSunTimes(doy);

    // Old system: PST (UTC-8) in winter, PDT (UTC-7) in summer
    const isDST_old = doy >= SPRING_FORWARD && doy < FALL_BACK;
    const offset_old = isDST_old ? 1 : 0;
    const sunrise_old = sun.sunrise + offset_old;
    const sunset_old = sun.sunset + offset_old;

    // New system: permanent PDT (UTC-7) all year
    const sunrise_new = sun.sunrise + 1;
    const sunset_new = sun.sunset + 1;

    // Figure out month/day for label
    let m = 1, d = doy;
    const daysInMonth = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for (let i = 0; i < 12; i++) {
      if (d <= daysInMonth[i]) { m = i + 1; break; }
      d -= daysInMonth[i];
    }

    data.push({
      doy,
      month: m,
      day: d,
      label: `${MONTHS[m - 1]} ${d}`,
      sunrise_old,
      sunset_old,
      sunrise_new,
      sunset_new,
      daylight_old: sunset_old - sunrise_old,
      daylight_new: sunset_new - sunrise_new,
      diff_sunrise: (sunrise_new - sunrise_old) * 60,
      diff_sunset: (sunset_new - sunset_old) * 60,
    });
  }
  return data;
}

const CustomTooltip = ({ active, payload }) => {
  if (!active || !payload?.length) return null;
  const d = payload[0]?.payload;
  if (!d) return null;

  return (
    <div style={{
      background: "rgba(15, 20, 35, 0.95)",
      border: "1px solid rgba(255,255,255,0.15)",
      borderRadius: 10,
      padding: "14px 18px",
      fontFamily: "'DM Sans', sans-serif",
      fontSize: 13,
      color: "#e0e0e0",
      lineHeight: 1.7,
      backdropFilter: "blur(10px)",
      minWidth: 220,
    }}>
      <div style={{ fontWeight: 700, fontSize: 15, marginBottom: 6, color: "#fff" }}>{d.label}</div>
      <div style={{ display: "grid", gridTemplateColumns: "auto 1fr 1fr", gap: "2px 14px" }}>
        <div></div>
        <div style={{ fontWeight: 600, color: "#7eb8da", fontSize: 11, textTransform: "uppercase", letterSpacing: 0.5 }}>Old</div>
        <div style={{ fontWeight: 600, color: "#f4a261", fontSize: 11, textTransform: "uppercase", letterSpacing: 0.5 }}>New (Perm DST)</div>
        <div style={{ color: "#aaa" }}>Sunrise</div>
        <div>{formatTime(d.sunrise_old)}</div>
        <div>{formatTime(d.sunrise_new)}</div>
        <div style={{ color: "#aaa" }}>Sunset</div>
        <div>{formatTime(d.sunset_old)}</div>
        <div>{formatTime(d.sunset_new)}</div>
      </div>
      {Math.abs(d.diff_sunrise) > 1 && (
        <div style={{ marginTop: 8, padding: "6px 10px", background: "rgba(255,255,255,0.06)", borderRadius: 6, fontSize: 12 }}>
          Winter difference: sunrise {Math.round(Math.abs(d.diff_sunrise))} min later, sunset {Math.round(Math.abs(d.diff_sunset))} min later
        </div>
      )}
    </div>
  );
};

const MonthTicks = [1, 32, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335];

function BCDSTComparison() {
  const [view, setView] = useState("sunset");
  const data = useMemo(generateData, []);

  const views = {
    sunrise: {
      title: "Sunrise Times",
      subtitle: "When the sun comes up on the clock",
      keys: [
        { key: "sunrise_old", color: "#5b9bd5", label: "Old (clock changes)" },
        { key: "sunrise_new", color: "#f4a261", label: "Permanent DST" },
      ],
      domain: [4, 10],
      note: "In winter, permanent DST means sunrise is ~1 hour later on the clock. Late December sunrise won't be until ~9 AM.",
    },
    sunset: {
      title: "Sunset Times",
      subtitle: "When the sun goes down on the clock",
      keys: [
        { key: "sunset_old", color: "#5b9bd5", label: "Old (clock changes)" },
        { key: "sunset_new", color: "#f4a261", label: "Permanent DST" },
      ],
      domain: [15.5, 21.5],
      note: "The big win: in winter, sunsets are ~1 hour later. December sunsets around 5:15 PM instead of 4:15 PM.",
    },
    both: {
      title: "Sunrise & Sunset",
      subtitle: "Full daylight envelope comparison",
      keys: [
        { key: "sunrise_old", color: "#5b9bd5", label: "Old sunrise" },
        { key: "sunrise_new", color: "#f4a261", label: "New sunrise" },
        { key: "sunset_old", color: "#7eb8da", label: "Old sunset" },
        { key: "sunset_new", color: "#e9c46a", label: "New sunset" },
      ],
      domain: [4, 22],
      note: "Total daylight hours don't change — the window just shifts 1 hour later in winter.",
    },
  };

  const v = views[view];

  // Key dates to highlight
  const winterSolstice = dayOfYear(12, 21);
  const summerSolstice = dayOfYear(6, 21);

  return (
    <div style={{
      minHeight: "100vh",
      background: "linear-gradient(170deg, #0a0e1a 0%, #121830 40%, #1a1a2e 100%)",
      color: "#e8e8e8",
      fontFamily: "'DM Sans', sans-serif",
      padding: "0 0 40px",
    }}>
      <link href="https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;600;700&family=Playfair+Display:wght@700;800&display=swap" rel="stylesheet" />

      {/* Header */}
      <div style={{
        padding: "48px 32px 32px",
        maxWidth: 960,
        margin: "0 auto",
      }}>
        <div style={{
          display: "inline-block",
          background: "linear-gradient(135deg, #f4a261, #e76f51)",
          color: "#fff",
          fontWeight: 700,
          fontSize: 11,
          letterSpacing: 1.5,
          textTransform: "uppercase",
          padding: "5px 14px",
          borderRadius: 20,
          marginBottom: 16,
        }}>
          Announced March 2, 2026
        </div>
        <h1 style={{
          fontFamily: "'Playfair Display', serif",
          fontSize: "clamp(28px, 5vw, 44px)",
          fontWeight: 800,
          lineHeight: 1.15,
          margin: "0 0 12px",
          background: "linear-gradient(135deg, #fff 30%, #a8c8e8)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
        }}>
          BC's Permanent<br />Daylight Saving Time
        </h1>
        <p style={{ color: "#8899bb", fontSize: 16, maxWidth: 520, lineHeight: 1.6, margin: 0 }}>
          How sunrise and sunset times in Vancouver will change when the clocks stop falling back after November 2026.
        </p>
      </div>

      {/* Key facts */}
      <div style={{
        maxWidth: 960,
        margin: "0 auto 32px",
        padding: "0 32px",
        display: "grid",
        gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
        gap: 16,
      }}>
        {[
          { label: "Last clock change", value: "Mar 8, 2026", icon: "\u{1F550}" },
          { label: "Dec 21 sunset (old)", value: "4:16 PM", icon: "\u{1F305}" },
          { label: "Dec 21 sunset (new)", value: "5:16 PM", icon: "\u{1F307}" },
          { label: "Dec 21 sunrise (new)", value: "9:04 AM", icon: "\u{1F304}" },
        ].map((item, i) => (
          <div key={i} style={{
            background: "rgba(255,255,255,0.04)",
            border: "1px solid rgba(255,255,255,0.08)",
            borderRadius: 14,
            padding: "18px 20px",
          }}>
            <div style={{ fontSize: 24, marginBottom: 6 }}>{item.icon}</div>
            <div style={{ fontSize: 11, color: "#7788aa", textTransform: "uppercase", letterSpacing: 0.8, marginBottom: 4, fontWeight: 600 }}>{item.label}</div>
            <div style={{ fontSize: 22, fontWeight: 700 }}>{item.value}</div>
          </div>
        ))}
      </div>

      {/* View toggle */}
      <div style={{
        maxWidth: 960,
        margin: "0 auto 8px",
        padding: "0 32px",
        display: "flex",
        gap: 8,
        flexWrap: "wrap",
      }}>
        {Object.entries(views).map(([key, val]) => (
          <button
            key={key}
            onClick={() => setView(key)}
            style={{
              background: view === key ? "rgba(244,162,97,0.2)" : "rgba(255,255,255,0.05)",
              border: view === key ? "1px solid rgba(244,162,97,0.5)" : "1px solid rgba(255,255,255,0.1)",
              color: view === key ? "#f4a261" : "#8899bb",
              borderRadius: 10,
              padding: "10px 20px",
              cursor: "pointer",
              fontSize: 14,
              fontWeight: 600,
              fontFamily: "'DM Sans', sans-serif",
              transition: "all 0.2s",
            }}
          >
            {val.title}
          </button>
        ))}
      </div>

      {/* Chart */}
      <div style={{
        maxWidth: 960,
        margin: "0 auto",
        padding: "16px 16px 0 0",
      }}>
        <div style={{ padding: "0 32px 8px", display: "flex", justifyContent: "space-between", alignItems: "baseline", flexWrap: "wrap", gap: 8 }}>
          <div>
            <h2 style={{ margin: 0, fontSize: 20, fontWeight: 700 }}>{v.title}</h2>
            <p style={{ margin: "2px 0 0", color: "#667799", fontSize: 13 }}>{v.subtitle}</p>
          </div>
          <div style={{ display: "flex", gap: 16, fontSize: 13 }}>
            {v.keys.map(k => (
              <div key={k.key} style={{ display: "flex", alignItems: "center", gap: 6 }}>
                <div style={{ width: 12, height: 3, borderRadius: 2, background: k.color }} />
                <span style={{ color: "#8899bb" }}>{k.label}</span>
              </div>
            ))}
          </div>
        </div>

        <ResponsiveContainer width="100%" height={380}>
          <AreaChart data={data} margin={{ top: 10, right: 20, bottom: 20, left: 10 }}>
            <defs>
              {v.keys.map(k => (
                <linearGradient key={k.key} id={`grad_${k.key}`} x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor={k.color} stopOpacity={0.2} />
                  <stop offset="100%" stopColor={k.color} stopOpacity={0.02} />
                </linearGradient>
              ))}
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.05)" />
            <XAxis
              dataKey="doy"
              ticks={MonthTicks}
              tickFormatter={(doy) => MONTHS[Math.min(11, MonthTicks.indexOf(doy) >= 0 ? MonthTicks.indexOf(doy) : 0)]}
              stroke="#445"
              tick={{ fill: "#667799", fontSize: 12 }}
              axisLine={{ stroke: "rgba(255,255,255,0.1)" }}
            />
            <YAxis
              domain={v.domain}
              tickFormatter={formatHourAxis}
              stroke="#445"
              tick={{ fill: "#667799", fontSize: 12 }}
              axisLine={{ stroke: "rgba(255,255,255,0.1)" }}
            />
            <Tooltip content={<CustomTooltip />} />

            {/* Spring forward / fall back markers */}
            <ReferenceLine x={SPRING_FORWARD} stroke="rgba(244,162,97,0.3)" strokeDasharray="4 4" label={{ value: "Spring fwd", fill: "#f4a26188", fontSize: 10, position: "top" }} />
            <ReferenceLine x={FALL_BACK} stroke="rgba(244,162,97,0.3)" strokeDasharray="4 4" label={{ value: "No fall back", fill: "#f4a26188", fontSize: 10, position: "top" }} />

            {v.keys.map(k => (
              <Area
                key={k.key}
                type="monotone"
                dataKey={k.key}
                stroke={k.color}
                strokeWidth={2.5}
                fill={`url(#grad_${k.key})`}
                dot={false}
                activeDot={{ r: 4, fill: k.color }}
              />
            ))}
          </AreaChart>
        </ResponsiveContainer>

        {/* Note */}
        <div style={{
          maxWidth: 960,
          margin: "0 auto",
          padding: "8px 32px 0",
        }}>
          <div style={{
            background: "rgba(244,162,97,0.08)",
            border: "1px solid rgba(244,162,97,0.15)",
            borderRadius: 12,
            padding: "14px 18px",
            fontSize: 14,
            color: "#c8b89a",
            lineHeight: 1.6,
          }}>
            {v.note}
          </div>
        </div>
      </div>

      {/* Explanation section */}
      <div style={{
        maxWidth: 960,
        margin: "40px auto 0",
        padding: "0 32px",
      }}>
        <h2 style={{ fontSize: 22, fontWeight: 700, marginBottom: 20, fontFamily: "'Playfair Display', serif" }}>
          How It Works
        </h2>
        <div style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fit, minmax(280px, 1fr))",
          gap: 20,
        }}>
          <div style={{
            background: "rgba(91,155,213,0.08)",
            border: "1px solid rgba(91,155,213,0.15)",
            borderRadius: 14,
            padding: "24px",
          }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: "#5b9bd5", textTransform: "uppercase", letterSpacing: 1, marginBottom: 10 }}>Old System</div>
            <div style={{ fontSize: 14, lineHeight: 1.8, color: "#99aabb" }}>
              <strong style={{ color: "#c8d8e8" }}>Mar &rarr; Nov:</strong> PDT (UTC-7) — clocks spring forward<br />
              <strong style={{ color: "#c8d8e8" }}>Nov &rarr; Mar:</strong> PST (UTC-8) — clocks fall back<br /><br />
              Winter sunsets as early as 4:15 PM.
              Sunrise around 8:05 AM at solstice.
            </div>
          </div>
          <div style={{
            background: "rgba(244,162,97,0.08)",
            border: "1px solid rgba(244,162,97,0.15)",
            borderRadius: 14,
            padding: "24px",
          }}>
            <div style={{ fontSize: 13, fontWeight: 700, color: "#f4a261", textTransform: "uppercase", letterSpacing: 1, marginBottom: 10 }}>New System (Nov 2026+)</div>
            <div style={{ fontSize: 14, lineHeight: 1.8, color: "#bbaa88" }}>
              <strong style={{ color: "#e8d8c8" }}>All year:</strong> Pacific Time (UTC-7) — no changes ever<br /><br />
              Winter sunsets around 5:15 PM — an hour later!<br />
              Trade-off: sunrise not until ~9:04 AM at solstice.
            </div>
          </div>
        </div>

        {/* Timeline */}
        <div style={{
          marginTop: 32,
          background: "rgba(255,255,255,0.03)",
          border: "1px solid rgba(255,255,255,0.08)",
          borderRadius: 14,
          padding: "24px 28px",
        }}>
          <div style={{ fontSize: 13, fontWeight: 700, color: "#8899bb", textTransform: "uppercase", letterSpacing: 1, marginBottom: 16 }}>Timeline</div>
          <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
            {[
              { date: "Mar 8, 2026", text: "Clocks spring forward (final time change)", active: true },
              { date: "Mar\u2013Oct 2026", text: "No difference — both systems are on UTC-7 during summer" },
              { date: "Nov 1, 2026", text: "Clocks DON'T fall back. This is when the change is felt.", active: true },
              { date: "Winter 2026\u201327", text: "First winter with later sunrises & later sunsets" },
            ].map((item, i) => (
              <div key={i} style={{ display: "flex", gap: 14, alignItems: "flex-start" }}>
                <div style={{
                  width: 10,
                  height: 10,
                  borderRadius: "50%",
                  background: item.active ? "#f4a261" : "rgba(255,255,255,0.15)",
                  marginTop: 5,
                  flexShrink: 0,
                }} />
                <div>
                  <span style={{ fontWeight: 700, color: item.active ? "#f4a261" : "#aabbcc", fontSize: 14 }}>{item.date}</span>
                  <span style={{ color: "#8899bb", fontSize: 14 }}> — {item.text}</span>
                </div>
              </div>
            ))}
          </div>
        </div>

        <p style={{ marginTop: 24, fontSize: 12, color: "#556677", lineHeight: 1.6 }}>
          Times are approximate for Vancouver, BC (49.3&deg;N). Actual times vary slightly by location within BC and by atmospheric conditions.
        </p>
      </div>
    </div>
  );
}

// Mount the app
createRoot(document.getElementById("root")).render(<BCDSTComparison />);
