// Marketplace data (Tools view). UI chrome stays in i18n; these entries
// are content. Apps = the real Rust tool registry; "soon" apps are
// honestly labeled as not yet supported.

export interface SoonApp {
  name: string;
  desc: string;
  /** i18n key of the category label (mkt.cat.*). */
  catKey: string;
  color: string;
  glyph: string;
}

/** Apps from the reference marketplace that Minime does not support yet.
 *  Shown honestly as "Segera hadir" (disabled) — never faked as working. */
export const SOON_APPS: SoonApp[] = [
  { name: "1Password", desc: "Masuk ke situs dengan login yang kamu bagikan.", catKey: "mkt.cat.productivity", color: "#1a73e8", glyph: "1P" },
  { name: "Bitwarden", desc: "Isi login browser dari Bitwarden.", catKey: "mkt.cat.productivity", color: "#175DDC", glyph: "Bw" },
  { name: "Aave", desc: "Data dan transaksi Aave.", catKey: "mkt.cat.data", color: "#B6509E", glyph: "Aa" },
  { name: "Linear", desc: "Issue dan triase proyek.", catKey: "mkt.cat.code", color: "#5E6AD2", glyph: "Ln" },
  { name: "Figma", desc: "Desain dan prototipe.", catKey: "mkt.cat.design", color: "#A259FF", glyph: "Fg" },
  { name: "Cobrain", desc: "Memori bersama untuk agen AImu.", catKey: "mkt.cat.productivity", color: "#E5484D", glyph: "Cb" },
  { name: "Paper", desc: "Kanvas desain berbasis HTML dan CSS.", catKey: "mkt.cat.design", color: "#0EA5E9", glyph: "Pa" },
  { name: "Sentry", desc: "Triase error dan crash.", catKey: "mkt.cat.code", color: "#6C2BD9", glyph: "Se" },
  { name: "Vercel", desc: "Deployment, build log, dan proyek.", catKey: "mkt.cat.code", color: "#111111", glyph: "Ve" },
  { name: "Context7", desc: "Dokumentasi library terkini.", catKey: "mkt.cat.research", color: "#16A34A", glyph: "C7" },
  { name: "Stripe", desc: "Tinjauan pembayaran dan penagihan.", catKey: "mkt.cat.other", color: "#635BFF", glyph: "St" },
  { name: "PostHog", desc: "Analitik produk dan feature flag.", catKey: "mkt.cat.data", color: "#F54E00", glyph: "PH" },
  { name: "Airtable", desc: "Basis data dan catatan.", catKey: "mkt.cat.data", color: "#2D7FF9", glyph: "At" },
  { name: "Firecrawl", desc: "Ekstraksi dan pencarian web.", catKey: "mkt.cat.research", color: "#EA580C", glyph: "Fc" },
  { name: "Brave Search", desc: "Pencarian web privat.", catKey: "mkt.cat.research", color: "#FB542B", glyph: "Bs" },
  { name: "Resend", desc: "Email transaksional.", catKey: "mkt.cat.other", color: "#000000", glyph: "Re" },
  { name: "Composio", desc: "Banyak aplikasi lewat tautan Composio-mu.", catKey: "mkt.cat.other", color: "#1D4ED8", glyph: "Co" },
];

/** Brand tile for the 12 real tools (matches the reference card layout). */
export const APP_TILES: Record<string, { color: string; glyph: string }> = {
  meta: { color: "#2563EB", glyph: "M" },
  postiz: { color: "#7C3AED", glyph: "Po" },
  pireel: { color: "#E11D48", glyph: "Pr" },
  drive: { color: "#059669", glyph: "D" },
  slack: { color: "#D97706", glyph: "S" },
  discord: { color: "#5865F2", glyph: "Dc" },
  telegram: { color: "#0891B2", glyph: "T" },
  github: { color: "#6B7280", glyph: "G" },
  notion: { color: "#A3A3A3", glyph: "N" },
  canva: { color: "#0EA5E9", glyph: "C" },
  ruangapi: { color: "#0EA5E9", glyph: "R" },
  browser: { color: "#4FA6F0", glyph: "B" },
  pixabay: { color: "#2EC66D", glyph: "Px" },
  ffmpeg: { color: "#1F2937", glyph: "Ff" },
  whisper: { color: "#7C3AED", glyph: "Wh" },
  arbotts: { color: "#EA580C", glyph: "Ar" },
  sdimage: { color: "#4FA6F0", glyph: "Sd" },
};

/** Backend category id -> i18n label key. */
export const APP_CAT_KEYS: Record<string, string> = {
  social: "mkt.cat.social",
  chat: "mkt.cat.chat",
  video: "mkt.cat.video",
  storage: "mkt.cat.productivity",
  code: "mkt.cat.code",
  docs: "mkt.cat.productivity",
  design: "mkt.cat.design",
  api: "mkt.cat.data",
  web: "mkt.cat.research",
  media: "mkt.cat.media",
};

/** Which tools have a per-language description key (tools.<id>Desc). */
export function appDescKey(id: string): string {
  return id === "browser" ? "mkt.desc.browser" : `tools.${id}Desc`;
}
