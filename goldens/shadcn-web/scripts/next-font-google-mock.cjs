// next/font/google offline build helper (Windows-friendly).
//
// Next.js downloads Google Fonts at build time. In offline / firewalled environments, `next build`
// fails. Next supports a test-only escape hatch via `NEXT_FONT_GOOGLE_MOCKED_RESPONSES`:
// https://github.com/vercel/next.js/blob/canary/packages/font/src/google/fetch-css-from-google-fonts.ts
//
// We return a deterministic @font-face CSS response for any requested URL, mapping requested font
// families to locally available Windows font files (Segoe UI + Consolas). This keeps the shadcn v4
// app buildable for golden extraction on Windows without external network access.

const DEFAULT_SANS_TTF = "//?/C:/Windows/Fonts/segoeui.ttf"
const DEFAULT_MONO_TTF = "//?/C:/Windows/Fonts/consola.ttf"

function parseFamilies(url) {
  try {
    const u = new URL(url)
    const families = u.searchParams.getAll("family")
    if (families.length === 0) return []
    return families
      .map((f) => String(f))
      .map((f) => f.split(":")[0] ?? f)
      .map((f) => f.replace(/\+/g, " "))
      .map((f) => f.trim())
      .filter(Boolean)
  } catch {
    return []
  }
}

function isMonoFamily(family) {
  const v = family.toLowerCase()
  return v.includes("mono") || v.includes("jetbrains")
}

function mockCssForFamily(family) {
  const src = isMonoFamily(family) ? DEFAULT_MONO_TTF : DEFAULT_SANS_TTF
  // Use a wide weight range so CSS using 400/500/600/700 still resolves to a self-hosted font file.
  // (The file itself is not variable; browsers will synthesize weights as needed.)
  return `@font-face {
  font-family: '${family}';
  font-style: normal;
  font-weight: 100 900;
  font-display: swap;
  src: url(${src}) format('truetype');
}`
}

module.exports = new Proxy(
  {},
  {
    get(_target, prop) {
      if (typeof prop !== "string") return undefined
      const families = parseFamilies(prop)
      if (families.length === 0) return undefined
      return families.map(mockCssForFamily).join("\n")
    },
  }
)

