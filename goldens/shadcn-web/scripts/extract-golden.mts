import fs from "fs"
import http from "http"
import os from "os"
import path from "path"
import type puppeteer from "puppeteer"
import { createRequire } from "module"
import { fileURLToPath, pathToFileURL } from "url"

type Theme = "light" | "dark"
type Mode = "closed" | "open"
type OpenAction = "click" | "hover" | "contextmenu" | "keys"

type OpenPoint = { abs: { x: number; y: number }; rel: { x: number; y: number } }
type OpenMeta = { action: OpenAction; selector: string; point: { x: number; y: number } }

type OpenVariant = { variant: string; selector: string }

type GoldenVariant = { variant: string }

type KeyChord = { modifiers: string[]; key: string }

type OpenStep =
  | { action: "wait"; waitMs: number }
  | { action: "waitFor"; selector: string }
  | { action: "move"; x: number; y: number }
  | { action: "tabTo"; selector: string; maxTabs: number }
  | { action: "attr"; selector: string; name: string; value: string }
  | { action: "mouseDown"; selector: string }
  | { action: "mouseUp"; selector: string }
  | { action: Exclude<OpenAction, "keys">; selector: string }
  | { action: "keys"; selector: string; keys: KeyChord[] }
  | { action: "type"; selector: string; text: string }
  | { action: "scroll"; selector: string; dx: number; dy: number }
  | { action: "scrollTo"; selector: string; left: number; top: number }

type GoldenOptions = {
  baseUrl: string
  style: string
  themes: Theme[]
  modes: Mode[]
  names: string[]
  types: string[]
  outDir: string
  update: boolean
  timeoutMs: number
  viewportW: number
  viewportH: number
  deviceScaleFactor: number
  freezeDate?: string
  openSelector?: string
  openAction?: OpenAction
  openKeys?: KeyChord
  steps?: OpenStep[]
  openSteps?: OpenStep[]
  openVariants?: OpenVariant[]
  variants?: GoldenVariant[]
  mergeThemes?: boolean
}

type GoldenFile = {
  version: number
  style: string
  name: string
  mode?: Mode
  variant?: string
  themes: Record<string, unknown>
}

function parseArgs(argv: string[]): {
  flags: Record<string, string | boolean>
  names: string[]
} {
  const flags: Record<string, string | boolean> = {}
  const names: string[] = []

  for (const arg of argv) {
    if (!arg.startsWith("--")) {
      names.push(arg)
      continue
    }

    const eq = arg.indexOf("=")
    if (eq === -1) {
      flags[arg.slice(2)] = true
      continue
    }

    const key = arg.slice(2, eq)
    const value = arg.slice(eq + 1)
    flags[key] = value
  }

  return { flags, names }
}

function round3(v: number) {
  return Math.round(v * 1000) / 1000
}

function ensureDir(dir: string) {
  fs.mkdirSync(dir, { recursive: true })
}

type StartedServer = {
  baseUrl: string
  close: () => Promise<void>
}

async function loadNext(nextDir: string) {
  const require = createRequire(import.meta.url)

  const candidates = [
    nextDir,
    path.join(repoRoot, "repo-ref", "ui", "apps", "v4"),
    path.join(repoRoot, "repo-ref", "ui"),
    repoRoot,
    process.cwd(),
  ]

  for (const candidate of candidates) {
    try {
      const entry = require.resolve("next", { paths: [candidate] })
      const mod = await import(pathToFileURL(entry).href)
      return (mod as any).default ?? mod
    } catch {
      // keep searching
    }
  }

  const mod = await import("next")
  return (mod as any).default ?? mod
}

async function waitForHttpOk(url: string, timeoutMs: number) {
  const start = Date.now()
  let lastError: unknown = null

  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url)
      if (res.ok) return
      lastError = new Error(`status ${res.status} for ${url}`)
    } catch (err) {
      lastError = err
    }
    await new Promise((r) => setTimeout(r, 200))
  }

  throw new Error(
    `timed out waiting for ${url} (${timeoutMs}ms; lastError=${String(lastError)})`
  )
}

async function startNextServer(
  nextDir: string,
  baseUrl: string
): Promise<StartedServer> {
  const debug = process.env.DEBUG_GOLDENS === "1"
  // Some Next.js plugins/configs (e.g. docs tooling) resolve files relative to `process.cwd()`.
  // When we run the extractor from the repo root, those lookups can incorrectly point at the
  // workspace root instead of the Next app directory (e.g. resolving `source.config.ts`).
  //
  // We temporarily `chdir` so Next (and its plugins) see the expected project root.
  const prevCwd = process.cwd()
  process.chdir(nextDir)
  if (debug) {
    console.log(`- startNextServer: chdir ${prevCwd} -> ${process.cwd()}`)
  }

  const url = new URL(baseUrl)
  const hostname = url.hostname || "localhost"
  const port = url.port ? Number(url.port) : url.protocol === "https:" ? 443 : 80

  process.env.NODE_ENV = "production"
  process.env.NEXT_PUBLIC_APP_URL = baseUrl
  process.env.PORT = String(port)

  const next = await loadNext(nextDir)
  const app = next({ dev: false, dir: nextDir, hostname, port })
  const handle = app.getRequestHandler()
  await app.prepare()

  const server = http.createServer((req, res) => handle(req, res))

  await new Promise<void>((resolve, reject) => {
    server.once("error", reject)
    server.listen(port, hostname, () => resolve())
  })
  if (debug) {
    console.log(`- startNextServer: listening; cwd=${process.cwd()}`)
  }

  return {
    baseUrl,
    close: async () => {
      try {
        await new Promise<void>((resolve) => server.close(() => resolve()))
        if (typeof app?.close === "function") {
          await app.close()
        }
      } finally {
        if (debug) {
          console.log(`- startNextServer: restore cwd ${process.cwd()} -> ${prevCwd}`)
        }
        process.chdir(prevCwd)
      }
    },
  }
}

function parseOpenVariants(raw: string): OpenVariant[] {
  const parts = raw
    .split(";")
    .map((p) => p.trim())
    .filter(Boolean)

  const out: OpenVariant[] = []
  for (const part of parts) {
    const eq = part.indexOf("=")
    if (eq === -1) {
      throw new Error(
        `invalid --openVariants entry "${part}" (expected "<variant>=<css>")`
      )
    }
    const variant = part.slice(0, eq).trim()
    const selector = part.slice(eq + 1).trim()
    if (!variant) {
      throw new Error(`invalid --openVariants entry "${part}" (empty variant)`)
    }
    if (!/^[a-zA-Z0-9][a-zA-Z0-9_-]*$/.test(variant)) {
      throw new Error(
        `invalid --openVariants variant "${variant}" (expected [a-zA-Z0-9][a-zA-Z0-9_-]*)`
      )
    }
    if (!selector) {
      throw new Error(
        `invalid --openVariants entry "${part}" (empty selector for variant=${variant})`
      )
    }
    out.push({ variant, selector })
  }

  return out
}

function parseVariants(raw: string): GoldenVariant[] {
  const parts = raw
    .split(";")
    .map((p) => p.trim())
    .filter(Boolean)

  const out: GoldenVariant[] = []
  for (const part of parts) {
    const variant = part.trim()
    if (!variant) {
      throw new Error(`invalid --variants entry "${part}" (empty variant)`)
    }
    if (!/^[a-zA-Z0-9][a-zA-Z0-9_-]*$/.test(variant)) {
      throw new Error(
        `invalid --variants variant "${variant}" (expected [a-zA-Z0-9][a-zA-Z0-9_-]*)`
      )
    }
    out.push({ variant })
  }

  return out
}

function normalizeKeyToken(raw: string): string {
  const v = raw.trim()
  if (!v) return v

  const lower = v.toLowerCase()
  if (lower === "ctrl" || lower === "control") return "Control"
  if (lower === "cmd" || lower === "command" || lower === "meta" || lower === "win")
    return "Meta"
  if (lower === "alt" || lower === "option") return "Alt"
  if (lower === "shift") return "Shift"

  // Let callers pass Puppeteer key codes directly (e.g. "KeyJ", "Digit1", "F10").
  if (/^(Key[A-Z]|Digit[0-9]|F[0-9]{1,2}|Arrow(Up|Down|Left|Right))$/.test(v)) return v

  if (/^[a-zA-Z]$/.test(v)) return `Key${v.toUpperCase()}`
  if (/^[0-9]$/.test(v)) return `Digit${v}`

  // Common named keys.
  if (
    [
      "Enter",
      "Escape",
      "Tab",
      "Backspace",
      "Delete",
      "Space",
      "Home",
      "End",
      "PageUp",
      "PageDown",
    ].includes(v)
  ) {
    return v
  }

  throw new Error(`invalid key token "${raw}"`)
}

function parseOpenKeys(raw: string): KeyChord {
  const parts = raw
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean)

  if (parts.length === 0) {
    throw new Error(`invalid --openKeys="${raw}" (empty chord)`)
  }

  const norm = parts.map(normalizeKeyToken)
  const key = norm[norm.length - 1]
  const modifiers = norm.slice(0, -1)

  for (const m of modifiers) {
    if (m !== "Shift" && m !== "Control" && m !== "Alt" && m !== "Meta") {
      throw new Error(
        `invalid --openKeys="${raw}" (modifier "${m}" must be Shift|Control|Alt|Meta)`
      )
    }
  }

  // De-dupe modifiers while preserving order.
  const dedupedMods: string[] = []
  for (const m of modifiers) {
    if (!dedupedMods.includes(m)) dedupedMods.push(m)
  }

  return { modifiers: dedupedMods, key }
}

function parseKeySequence(raw: string): KeyChord[] {
  const v = raw.trim()
  if (!v) {
    throw new Error(`invalid key sequence "${raw}" (empty)`)
  }

  // Treat any "+" as a chord spec (e.g. "Shift+F10", "Control+KeyJ").
  if (v.includes("+")) {
    return [parseOpenKeys(v)]
  }

  const tokens = v
    .split(/[,\s]+/)
    .map((p) => p.trim())
    .filter(Boolean)

  if (tokens.length === 0) {
    throw new Error(`invalid key sequence "${raw}" (empty)`)
  }

  return tokens.map((t) => ({ modifiers: [], key: normalizeKeyToken(t) }))
}

function parseOpenSteps(raw: string, openKeys: KeyChord | undefined): OpenStep[] {
  const parts = raw
    .split(";")
    .map((p) => p.trim())
    .filter(Boolean)

  const out: OpenStep[] = []
  for (const part of parts) {
    const eq = part.indexOf("=")
    if (eq === -1) {
      throw new Error(
        `invalid --openSteps entry "${part}" (expected "<action>=<value>")`
      )
    }

    const actionRaw = part.slice(0, eq).trim()
    const valueRaw = part.slice(eq + 1).trim()

    if (actionRaw === "wait") {
      const waitMs = Number(valueRaw)
      if (!Number.isFinite(waitMs) || waitMs < 0) {
        throw new Error(
          `invalid --openSteps wait="${valueRaw}" (expected non-negative ms)`
        )
      }
      out.push({ action: "wait", waitMs })
      continue
    }

    if (actionRaw === "waitFor") {
      if (!valueRaw) {
        throw new Error(`invalid --openSteps entry "${part}" (empty selector for waitFor)`)
      }
      out.push({ action: "waitFor", selector: valueRaw })
      continue
    }

    if (actionRaw === "move") {
      const comma = valueRaw.indexOf(",")
      if (comma === -1) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "move=<x>,<y>")`
        )
      }
      const xRaw = valueRaw.slice(0, comma).trim()
      const yRaw = valueRaw.slice(comma + 1).trim()
      const x = Number(xRaw)
      const y = Number(yRaw)
      if (!Number.isFinite(x) || !Number.isFinite(y)) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected finite x,y numbers)`
        )
      }
      out.push({ action: "move", x, y })
      continue
    }

    if (!valueRaw) {
      throw new Error(
        `invalid --openSteps entry "${part}" (empty value for action=${actionRaw})`
      )
    }

    if (actionRaw === "tabTo") {
      const at = valueRaw.indexOf("@")
      const selector = (at === -1 ? valueRaw : valueRaw.slice(0, at)).trim()
      const maxTabsRaw = at === -1 ? "" : valueRaw.slice(at + 1).trim()
      if (!selector) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "tabTo=<selector>[@<maxTabs>]")`
        )
      }
      const maxTabs = maxTabsRaw ? Number(maxTabsRaw) : 20
      if (!Number.isFinite(maxTabs) || maxTabs <= 0) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected positive maxTabs for tabTo)`
        )
      }
      out.push({ action: "tabTo", selector, maxTabs })
      continue
    }

    if (actionRaw === "keys") {
      let selector = valueRaw
      let keysSpec: string | undefined

      const at = valueRaw.indexOf("@")
      if (at !== -1) {
        selector = valueRaw.slice(0, at).trim()
        keysSpec = valueRaw.slice(at + 1).trim()
        if (!selector || !keysSpec) {
          throw new Error(
            `invalid --openSteps entry "${part}" (expected "keys=<selector>@<keys>")`
          )
        }
      }

      const keys = keysSpec
        ? parseKeySequence(keysSpec)
        : openKeys
          ? [openKeys]
          : null

      if (!keys) {
        throw new Error(
          `invalid --openSteps entry "${part}" (action=keys requires "<selector>@<keys>" or --openKeys=... / OPEN_KEYS=...)`
        )
      }

      out.push({ action: "keys", selector, keys })
      continue
    }

    if (actionRaw === "scroll") {
      const at = valueRaw.indexOf("@")
      if (at === -1) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "scroll=<selector>@<dx>,<dy>")`
        )
      }
      const selector = valueRaw.slice(0, at).trim()
      const deltaRaw = valueRaw.slice(at + 1).trim()
      if (!selector || !deltaRaw) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "scroll=<selector>@<dx>,<dy>")`
        )
      }
      const comma = deltaRaw.indexOf(",")
      if (comma === -1) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "scroll=<selector>@<dx>,<dy>")`
        )
      }
      const dxRaw = deltaRaw.slice(0, comma).trim()
      const dyRaw = deltaRaw.slice(comma + 1).trim()
      const dx = Number(dxRaw)
      const dy = Number(dyRaw)
      if (!Number.isFinite(dx) || !Number.isFinite(dy)) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected finite dx,dy numbers)`
        )
      }
      out.push({ action: "scroll", selector, dx, dy })
      continue
    }

    if (actionRaw === "scrollTo") {
      const at = valueRaw.indexOf("@")
      if (at === -1) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "scrollTo=<selector>@<left>,<top>")`
        )
      }
      const selector = valueRaw.slice(0, at).trim()
      const posRaw = valueRaw.slice(at + 1).trim()
      if (!selector || !posRaw) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "scrollTo=<selector>@<left>,<top>")`
        )
      }
      const comma = posRaw.indexOf(",")
      if (comma === -1) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "scrollTo=<selector>@<left>,<top>")`
        )
      }
      const leftRaw = posRaw.slice(0, comma).trim()
      const topRaw = posRaw.slice(comma + 1).trim()
      const left = Number(leftRaw)
      const top = Number(topRaw)
      if (!Number.isFinite(left) || !Number.isFinite(top)) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected finite left,top numbers)`
        )
      }
      out.push({ action: "scrollTo", selector, left, top })
      continue
    }

    if (actionRaw === "type") {
      const at = valueRaw.indexOf("@")
      if (at === -1) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "type=<selector>@<text>")`
        )
      }
      const selector = valueRaw.slice(0, at).trim()
      const text = valueRaw.slice(at + 1)
      if (!selector) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "type=<selector>@<text>")`
        )
      }
      out.push({ action: "type", selector, text })
      continue
    }

    if (actionRaw === "attr") {
      const at = valueRaw.indexOf("@")
      if (at === -1) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "attr=<selector>@<name>=<value>")`
        )
      }
      const selector = valueRaw.slice(0, at).trim()
      const rest = valueRaw.slice(at + 1).trim()
      if (!selector || !rest) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "attr=<selector>@<name>=<value>")`
        )
      }
      const eq2 = rest.indexOf("=")
      if (eq2 === -1) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "attr=<selector>@<name>=<value>")`
        )
      }
      const name = rest.slice(0, eq2).trim()
      const value = rest.slice(eq2 + 1)
      if (!name) {
        throw new Error(
          `invalid --openSteps entry "${part}" (expected "attr=<selector>@<name>=<value>")`
        )
      }
      out.push({ action: "attr", selector, name, value })
      continue
    }

    if (actionRaw === "mouseDown") {
      if (!valueRaw) {
        throw new Error(`invalid --openSteps entry "${part}" (empty selector for mouseDown)`)
      }
      out.push({ action: "mouseDown", selector: valueRaw })
      continue
    }

    if (actionRaw === "mouseUp") {
      if (!valueRaw) {
        throw new Error(`invalid --openSteps entry "${part}" (empty selector for mouseUp)`)
      }
      out.push({ action: "mouseUp", selector: valueRaw })
      continue
    }

    if (
      actionRaw !== "click" &&
      actionRaw !== "hover" &&
      actionRaw !== "contextmenu"
    ) {
      throw new Error(
        `invalid --openSteps action "${actionRaw}" (expected click|hover|contextmenu|keys|type|scroll|scrollTo|attr|mouseDown|mouseUp|wait|waitFor|move)`
      )
    }

    out.push({ action: actionRaw, selector: valueRaw } as OpenStep)
  }

  return out
}

function resolveBrowserExecutablePath(): string | undefined {
  const envPath = process.env.PUPPETEER_EXECUTABLE_PATH
  if (envPath && fs.existsSync(envPath)) {
    return envPath
  }

  // Prefer Puppeteer's managed browser by default for protocol compatibility.
  // If you want to force a system browser, set PUPPETEER_EXECUTABLE_PATH explicitly.
  return undefined
}

function writeIfChanged(filePath: string, json: unknown, update: boolean) {
  const data = JSON.stringify(json, null, 2) + "\n"

  if (fs.existsSync(filePath) && !update) {
    return
  }

  if (fs.existsSync(filePath)) {
    const existing = fs.readFileSync(filePath, "utf8")
    if (existing === data) {
      return
    }
  }

  fs.writeFileSync(filePath, data, "utf8")
}

async function extractOne(page: puppeteer.Page) {
  // NOTE: `tsx`/esbuild can inject `__name(...)` helpers when serializing functions in dev,
  // which breaks `page.evaluate(() => ...)` because the page context doesn't have `__name`.
  // Using a string expression avoids that entire class of issues.
  const expr = `(() => {
    const root =
      document.querySelector("[data-fret-golden-target]") ||
      document.querySelector("[data-fret-golden-root]") ||
      document.body;

    const rootRect = root.getBoundingClientRect();

    const activeElement =
      document.activeElement instanceof Element ? document.activeElement : null;
    const activeDescendantId =
      activeElement?.getAttribute("aria-activedescendant") || "";

    const attrKeys = [
      "role",
      "aria-label",
      "aria-labelledby",
      "aria-describedby",
      "aria-live",
      "aria-relevant",
      "aria-atomic",
      "aria-invalid",
      "aria-checked",
      "aria-selected",
      "aria-expanded",
      "aria-pressed",
      "aria-controls",
      "aria-activedescendant",
      "aria-disabled",
      "aria-hidden",
      "data-state",
      "data-disabled",
      "data-orientation",
      "data-side",
      "data-align",
      "data-slot",
      "data-variant",
      "data-size",
      // Minimal Radix attrs needed for stable ScrollArea matching.
      "data-radix-scroll-area-root",
      "data-radix-scroll-area-viewport",
      "data-radix-scroll-area-scrollbar",
      "data-radix-scroll-area-thumb",
      "data-radix-scroll-area-corner",
      // Useful for overlay wrapper alignment.
      "data-radix-popper-content-wrapper",
      // Sonner (toast) attributes.
      "data-sonner-toaster",
      "data-sonner-toast",
      "data-x-position",
      "data-y-position",
      "data-type",
    ];

    const styleKeys = [
      "display",
      "position",
      "boxSizing",
      "overflowX",
      "overflowY",
      "whiteSpace",
      "flexDirection",
      "flexWrap",
      "justifyContent",
      "alignItems",
      "alignSelf",
      "gap",
      "rowGap",
      "columnGap",
      "flexGrow",
      "flexShrink",
      "flexBasis",
      "width",
      "height",
      "minWidth",
      "minHeight",
      "maxWidth",
      "maxHeight",
      "paddingTop",
      "paddingRight",
      "paddingBottom",
      "paddingLeft",
      "marginTop",
      "marginRight",
      "marginBottom",
      "marginLeft",
      "borderTopWidth",
      "borderRightWidth",
      "borderBottomWidth",
      "borderLeftWidth",
      "borderTopLeftRadius",
      "borderTopRightRadius",
      "borderBottomRightRadius",
      "borderBottomLeftRadius",
      "borderTopColor",
      "borderRightColor",
      "borderBottomColor",
      "borderLeftColor",
      "backgroundColor",
      "color",
      "opacity",
      "boxShadow",
      "fontFamily",
      "fontSize",
      "fontWeight",
      "lineHeight",
      "letterSpacing",
      "textAlign",
      "textTransform",
    ];

    const collectAttrs = (el) => {
      const out = {};
      for (const key of attrKeys) {
        const v = el.getAttribute(key);
        if (v != null) out[key] = v;
      }
      // Keep id and class as dedicated fields for easier selectors, but we also preserve them in attrs.
      return out;
    };

    const collectStyle = (el) => {
      const cs = window.getComputedStyle(el);
      const out = {};
      for (const key of styleKeys) {
        out[key] = cs[key];
      }
      return out;
    };

    const collectText = (el) => {
      const txt = (el.textContent || "").trim();
      if (!txt) return null;
      if (txt.length > 200) return txt.slice(0, 200) + "…";
      return txt;
    };

    function areaOfRect(r) {
      if (!r) return 0;
      const w = typeof r.w === "number" ? r.w : typeof r.width === "number" ? r.width : 0;
      const h = typeof r.h === "number" ? r.h : typeof r.height === "number" ? r.height : 0;
      if (!isFinite(w) || !isFinite(h)) return 0;
      return Math.max(0, w) * Math.max(0, h);
    }

    function rectFromSvgBBox(el) {
      try {
        if (!(el instanceof SVGGraphicsElement)) return null;

        // Only apply SVG bbox transforms to Recharts nodes: this avoids churn for unrelated SVG
        // usage (icons, etc) while fixing getBoundingClientRect() returning near-zero boxes for
        // <g> layers in Radar/Radial charts.
        const cls = el.getAttribute("class") || "";
        if (!cls.includes("recharts-")) return null;

        const bbox = el.getBBox();
        const ctm = el.getScreenCTM();
        if (!ctm) return null;

        // Transform bbox corners into screen space; bbox is in local SVG units.
        const p1 = new DOMPoint(bbox.x, bbox.y).matrixTransform(ctm);
        const p2 = new DOMPoint(bbox.x + bbox.width, bbox.y).matrixTransform(ctm);
        const p3 = new DOMPoint(bbox.x, bbox.y + bbox.height).matrixTransform(ctm);
        const p4 = new DOMPoint(bbox.x + bbox.width, bbox.y + bbox.height).matrixTransform(ctm);

        const xs = [p1.x, p2.x, p3.x, p4.x];
        const ys = [p1.y, p2.y, p3.y, p4.y];

        const minX = Math.min(...xs);
        const maxX = Math.max(...xs);
        const minY = Math.min(...ys);
        const maxY = Math.max(...ys);

        const w = maxX - minX;
        const h = maxY - minY;
        if (!isFinite(w) || !isFinite(h) || w < 0 || h < 0) return null;

        return {
          x: minX - rootRect.x,
          y: minY - rootRect.y,
          w,
          h,
        };
      } catch {
        return null;
      }
    }

    function traverse(el, pathStr) {
      const rect = el.getBoundingClientRect();
      const bboxRect = rectFromSvgBBox(el);
      const rectArea = rect.width * rect.height;
      const bboxArea = areaOfRect(bboxRect);
      const useBbox =
        bboxRect &&
        // getBoundingClientRect() can report near-zero boxes for SVG <g> layers; prefer bbox when it
        // is meaningfully larger.
        (rectArea < 1 || bboxArea > rectArea * 4);
      const finalRect = useBbox ? bboxRect : null;
      const attrs = collectAttrs(el);
      const cls = el.getAttribute("class") || null;
      const id = el.getAttribute("id") || null;

      const out = {
        path: pathStr,
        tag: el.tagName.toLowerCase(),
        id: id || undefined,
        className: cls || undefined,
        attrs,
        rect: {
          x: finalRect ? finalRect.x : rect.x - rootRect.x,
          y: finalRect ? finalRect.y : rect.y - rootRect.y,
          w: finalRect ? finalRect.w : rect.width,
          h: finalRect ? finalRect.h : rect.height,
        },
        computedStyle: collectStyle(el),
        text: collectText(el) || undefined,
        active: document.activeElement === el,
        activeDescendant: Boolean(activeDescendantId && id === activeDescendantId),
        children: [],
      };

      // Scroll metrics are integer-valued in the DOM (scrollWidth, clientWidth, ...). We keep
      // them separate from rect (getBoundingClientRect() can be fractional) so non-web runtimes
      // can gate scroll range behavior 1:1.
      const slot = attrs["data-slot"];
      const role = attrs["role"];
      const isSelectViewport =
        el.hasAttribute("data-radix-select-viewport") || slot === "select-viewport";
      const isListbox =
        role === "listbox" &&
        (out.computedStyle.overflowX === "auto" ||
          out.computedStyle.overflowX === "scroll" ||
          out.computedStyle.overflowY === "auto" ||
          out.computedStyle.overflowY === "scroll");
      const isScrollViewport =
        el.hasAttribute("data-radix-scroll-area-viewport") ||
        slot === "scroll-area-viewport" ||
        isSelectViewport ||
        isListbox ||
        out.computedStyle.overflowX === "scroll" ||
        out.computedStyle.overflowY === "scroll";
      if (isScrollViewport) {
        out.scroll = {
          scrollWidth: el.scrollWidth,
          scrollHeight: el.scrollHeight,
          clientWidth: el.clientWidth,
          clientHeight: el.clientHeight,
          offsetWidth: el.offsetWidth,
          offsetHeight: el.offsetHeight,
          scrollLeft: el.scrollLeft,
          scrollTop: el.scrollTop,
        };
      }

      const children = Array.from(el.children);
      for (let i = 0; i < children.length; i++) {
        const child = children[i];
        out.children.push(traverse(child, pathStr ? pathStr + "." + i : "" + i));
      }
      return out;
    }

    const portalCandidates = [
      ...Array.from(
        document.querySelectorAll(
          "[data-state='open'],[data-state='delayed-open'],[data-state='instant-open']"
        )
      ),
      // Sonner does not use Radix-style data-state="open". Treat the toaster container as a
      // portal-ish surface only when it actually contains at least one toast.
      ...Array.from(document.querySelectorAll("[data-sonner-toaster]")).filter((el) =>
        el.querySelector("[data-sonner-toast]")
      ),
    ].filter((el) => !root.contains(el));

    // Prefer the most specific (leaf-most) open-state nodes so we capture the actual Radix content
    // element rather than wrappers/portals.
    const portalRoots = portalCandidates.filter(
      (el) => !portalCandidates.some((other) => other !== el && el.contains(other))
    );

    // For placement/geometry alignment we often need the positioned wrapper (Popper sets transform/top/left
    // on [data-radix-popper-content-wrapper]). Some Radix primitives (e.g. Select) render the actual content
    // element as position: relative inside the wrapper, so its rect is not suitable for placement checks.
    const rectContains = (outer, inner) => {
      const eps = 0.01;
      return (
        inner.x + eps >= outer.x &&
        inner.y + eps >= outer.y &&
        inner.x + inner.width <= outer.x + outer.width + eps &&
        inner.y + inner.height <= outer.y + outer.height + eps
      );
    };

    const portalWrapperRoots = portalRoots.map((el) => {
      const byAttr = el.closest("[data-radix-popper-content-wrapper]");
      if (byAttr && !root.contains(byAttr)) return byAttr;

      const leafRect = el.getBoundingClientRect();
      let best = null;
      let bestArea = Infinity;

      let cur = el;
      while (cur && cur instanceof Element && cur !== document.body) {
        if (root.contains(cur)) break;

        const cs = getComputedStyle(cur);
        const positioned =
          cs.position === "fixed" ||
          cs.position === "absolute" ||
          (cs.transform && cs.transform !== "none") ||
          (cs.translate && cs.translate !== "none");

        if (positioned) {
          const r = cur.getBoundingClientRect();
          if (rectContains(r, leafRect)) {
            const area = r.width * r.height;
            if (area < bestArea) {
              bestArea = area;
              best = cur;
            }
          }
        }

        cur = cur.parentElement;
      }

      return best || el;
    });

    const portals = portalRoots.map((el, idx) =>
      traverse(el, "portal." + idx)
    );

    const portalWrappers = portalWrapperRoots.map((el, idx) =>
      traverse(el, "portalWrapper." + idx)
    );

    return {
      url: location.href,
      devicePixelRatio: window.devicePixelRatio,
      viewport: { w: window.innerWidth, h: window.innerHeight },
      root: traverse(root, ""),
      portals,
      portalWrappers,
    };
  })()`

  return await page.evaluate(expr)
}

async function openOverlay(
  page: puppeteer.Page,
  name: string,
  timeoutMs: number,
  openSelector: string | undefined,
  openAction: OpenAction,
  openKeys: KeyChord | undefined
) {
  const rootSel = "[data-fret-golden-target]"
  const debug = process.env.DEBUG_GOLDENS === "1"

  async function resolveOpenPoint(sel: string): Promise<OpenPoint | null> {
    const expr = `(() => {
      const rootSel = ${JSON.stringify(rootSel)};
      const sel = ${JSON.stringify(sel)};

      const root = document.querySelector(rootSel) || document.body;
      const el = document.querySelector(sel);
      if (!el || !(el instanceof Element)) return null;

      el.scrollIntoView({ block: "center", inline: "center" });
      const rootRect = root.getBoundingClientRect();
      const r = el.getBoundingClientRect();
      const x = r.x + r.width / 2;
      const y = r.y + r.height / 2;

      return {
        abs: { x, y },
        rel: { x: x - rootRect.x, y: y - rootRect.y },
      };
    })()`
    return (await page.evaluate(expr)) as OpenPoint | null
  }

  const selectorCandidates: string[] = []
  if (openSelector) {
    selectorCandidates.push(openSelector)
    if (!openSelector.includes("[data-fret-golden-target]")) {
      selectorCandidates.push(`${rootSel} ${openSelector}`)
    }
  }

  if (openAction === "contextmenu") {
    selectorCandidates.push(
      `${rootSel} [data-slot='context-menu-trigger']`,
      `${rootSel} [data-slot='context-menu'] [data-slot$='trigger']`,
      `${rootSel} [data-slot$='trigger']`,
      `${rootSel}`
    )
  } else if (openAction === "hover") {
    selectorCandidates.push(
      `${rootSel} [data-slot='tooltip-trigger']`,
      `${rootSel} [data-slot='hover-card-trigger']`,
      `${rootSel} [data-slot$='trigger']`,
      `${rootSel} button`,
      `${rootSel}`
    )
  } else if (openAction === "keys") {
    // Keyboard-triggered pages may not render a clickable trigger (e.g. `command-dialog`).
    // Focus a stable node so the page receives key events.
    selectorCandidates.push(`${rootSel}`)
  } else {
    selectorCandidates.push(
      `${rootSel} [role='combobox'][aria-expanded='false']`,
      `${rootSel} [aria-haspopup='menu'][data-state='closed']`,
      `${rootSel} [aria-haspopup='menu']`,
      `${rootSel} [data-state='closed'][aria-haspopup]`,
      `${rootSel} button[data-state='closed']`,
      `${rootSel} button`
    )
  }

  const waitExpr = `(() => {
    const root = document.querySelector("${rootSel}") || document.body;
    ${
      name.startsWith("navigation-menu-demo")
        ? `
    // NavigationMenu does not portal its viewport by default, so "open" state stays within the
    // golden root. Treat an open viewport/content as success for open-mode extraction.
    if (root.querySelector("[data-slot='navigation-menu-viewport'][data-state='open']")) return true;
    if (root.querySelector("[data-slot='navigation-menu-content'][data-state='open']")) return true;
    `
        : ""
    }
    const outside = (sel) =>
      Array.from(document.querySelectorAll(sel)).filter((el) => !root.contains(el));

    if (outside("[data-state='open']").length > 0) return true;
    if (outside("[data-radix-popper-content-wrapper]").length > 0) return true;
    if (outside("[role='menu']").length > 0) return true;
    if (outside("[role='listbox']").length > 0) return true;
    if (outside("[role='dialog']").length > 0) return true;
    if (outside("[data-sonner-toast]").length > 0) return true;
    return false;
  })()`

  async function tryCloseOverlays() {
    try {
      await page.keyboard.press("Escape")
    } catch {
      // ignore
    }
  }

  // Use Puppeteer's input (trusted pointer events). Many Radix triggers listen on pointerdown;
  // calling `element.click()` from within `page.evaluate(...)` is not sufficient.
  let opened = false
  let lastError: unknown = null
  let openedSelector: string | null = null
  let openedPoint: OpenPoint | null = null

  const openBudgetMs = Math.min(timeoutMs, 2500)
  for (const sel of selectorCandidates) {
    try {
      const point = await resolveOpenPoint(sel)
      if (!point) {
        continue
      }

      if (debug) {
        console.log(
          `- openOverlay: ${name} try ${openAction} on ${sel} (abs=${point.abs.x.toFixed(
            1
          )},${point.abs.y.toFixed(1)})`
        )
      }

      if (openAction === "hover") {
        await page.mouse.move(point.abs.x, point.abs.y, { steps: 4 })
      } else if (openAction === "contextmenu") {
        await page.mouse.click(point.abs.x, point.abs.y, {
          button: "right",
          delay: 10,
        })
      } else if (openAction === "keys") {
        await page.focus(sel)
        if (openKeys) {
          for (const mod of openKeys.modifiers) {
            await page.keyboard.down(mod)
          }
          await page.keyboard.press(openKeys.key)
          for (const mod of [...openKeys.modifiers].reverse()) {
            await page.keyboard.up(mod)
          }
        } else {
          // Default key open policy: "Shift+F10" (common cross-platform context menu shortcut).
          await page.keyboard.down("Shift")
          await page.keyboard.press("F10")
          await page.keyboard.up("Shift")
        }
      } else {
        await page.mouse.click(point.abs.x, point.abs.y, {
          button: "left",
          delay: 10,
        })
      }

      await waitForExpr(page, waitExpr, openBudgetMs)

      opened = true
      openedSelector = sel
      openedPoint = point
      break
    } catch (err) {
      lastError = err
      await tryCloseOverlays()
    }
  }

  if (!opened || !openedSelector || !openedPoint) {
    throw new Error(
      `failed to open overlay for ${name}: no trigger worked (lastError=${String(
        lastError
      )})`
    )
  }

  if (debug) {
    console.log(`- openOverlay: ${name} opened (${openAction}) via ${openedSelector}`)
  }

  // Let open-state layout settle (portal mount + popper positioning).
  await waitForFonts(page, Math.min(2000, timeoutMs))
  await waitForExpr(page, waitExpr, timeoutMs)

  return {
    action: openAction,
    selector: openedSelector,
    point: { x: openedPoint.rel.x, y: openedPoint.rel.y },
  } satisfies OpenMeta
}

async function openOverlayOutsideCounts(
  page: puppeteer.Page,
  rootSel: string
): Promise<{
  popperWrapper: number
  roleMenu: number
  roleDialog: number
  dataStateOpen: number
  dataSonnerToast: number
}> {
  const expr = `(() => {
    const root = document.querySelector(${JSON.stringify(rootSel)}) || document.body;
    const outsideCount = (sel) =>
      Array.from(document.querySelectorAll(sel)).filter((el) => !root.contains(el)).length;

    return {
      popperWrapper: outsideCount("[data-radix-popper-content-wrapper]"),
      roleMenu: outsideCount("[role='menu']"),
      roleDialog: outsideCount("[role='dialog']"),
      dataStateOpen: outsideCount("[data-state='open']"),
      dataSonnerToast: outsideCount("[data-sonner-toast]"),
    };
  })()`
  return (await page.evaluate(expr)) as {
    popperWrapper: number
    roleMenu: number
    roleDialog: number
    dataStateOpen: number
    dataSonnerToast: number
  }
}

async function applySteps(
  page: puppeteer.Page,
  name: string,
  timeoutMs: number,
  steps: OpenStep[],
  rootSel: string
) {
  const debug = process.env.DEBUG_GOLDENS === "1"

  for (const [idx, step] of steps.entries()) {
    if (step.action === "wait") {
      if (debug) console.log(`- steps: ${name} step[${idx}] wait ${step.waitMs}ms`)
      await new Promise((r) => setTimeout(r, step.waitMs))
      continue
    }

    if (step.action === "waitFor") {
      if (debug) console.log(`- steps: ${name} step[${idx}] waitFor ${step.selector}`)
      await page.waitForSelector(step.selector, { timeout: Math.min(timeoutMs, 30000) })
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    if (step.action === "move") {
      if (debug) console.log(`- steps: ${name} step[${idx}] move ${step.x},${step.y}`)
      await page.mouse.move(step.x, step.y, { steps: 4 })
      continue
    }
    if (step.action === "tabTo") {
      if (debug) {
        console.log(
          `- steps: ${name} step[${idx}] tabTo ${step.selector} (maxTabs=${step.maxTabs})`
        )
      }
      await page.evaluate(() => {
        const active = document.activeElement
        if (active instanceof HTMLElement) active.blur()
      })
      for (let attempt = 0; attempt < step.maxTabs; attempt++) {
        const ok = (await page.evaluate((selector) => {
          const el = document.activeElement
          return el instanceof Element && el.matches(selector)
        }, step.selector)) as boolean
        if (ok) break
        await page.keyboard.press("Tab")
      }
      const ok = (await page.evaluate((selector) => {
        const el = document.activeElement
        return el instanceof Element && el.matches(selector)
      }, step.selector)) as boolean
      if (!ok) {
        throw new Error(
          `steps failed for ${name}: tabTo did not reach ${step.selector} within ${step.maxTabs} tabs`
        )
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }
    if (step.action === "scroll") {
      const ok = await page.evaluate(
        async ({ selector, dx, dy }) => {
          const raf2 = () =>
            new Promise<void>((r) =>
              requestAnimationFrame(() => requestAnimationFrame(() => r()))
            )

          const els = Array.from(document.querySelectorAll(selector)).filter(
            (e): e is Element => e instanceof Element
          )
          if (els.length === 0) return false

          const scrollables = els.filter((e) => typeof (e as any).scrollBy === "function")
          if (scrollables.length === 0) return false

          const clamp = (v: number, min: number, max: number) =>
            Math.max(min, Math.min(max, v))

          // Prefer the most "meaningfully scrollable" match (helps when a selector matches both
          // the intended viewport and a wrapper that doesn't actually scroll).
          let best: Element = scrollables[0]
          let bestScore = -1
          for (const el of scrollables) {
            const anyEl = el as any
            const scrollableX = Math.max(0, (anyEl.scrollWidth ?? 0) - (anyEl.clientWidth ?? 0))
            const scrollableY = Math.max(0, (anyEl.scrollHeight ?? 0) - (anyEl.clientHeight ?? 0))
            const score = scrollableX + scrollableY
            if (score > bestScore) {
              bestScore = score
              best = el
            }
          }

          const el = best as any

          // Radix Select (and a few other portals) can update the scroll range after open via
          // layout effects. Wait for the scroll range to stabilize before applying deltas so the
          // desired scroll doesn't get clamped to an early, smaller maxTop/maxLeft.
          let lastMaxLeft = -1
          let lastMaxTop = -1
          let stableFrames = 0
          for (let i = 0; i < 24; i++) {
            const maxLeft = Math.max(
              0,
              (Number(el.scrollWidth ?? 0) || 0) - (Number(el.clientWidth ?? 0) || 0)
            )
            const maxTop = Math.max(
              0,
              (Number(el.scrollHeight ?? 0) || 0) - (Number(el.clientHeight ?? 0) || 0)
            )
            if (maxLeft === lastMaxLeft && maxTop === lastMaxTop) {
              stableFrames++
              if (stableFrames >= 2) break
            } else {
              stableFrames = 0
              lastMaxLeft = maxLeft
              lastMaxTop = maxTop
            }
            await raf2()
          }

          const beforeLeft = Number(el.scrollLeft ?? 0) || 0
          const beforeTop = Number(el.scrollTop ?? 0) || 0
          const maxLeft = Math.max(
            0,
            (Number(el.scrollWidth ?? 0) || 0) - (Number(el.clientWidth ?? 0) || 0)
          )
          const maxTop = Math.max(
            0,
            (Number(el.scrollHeight ?? 0) || 0) - (Number(el.clientHeight ?? 0) || 0)
          )
          const desiredLeft = clamp(beforeLeft + dx, 0, maxLeft)
          const desiredTop = clamp(beforeTop + dy, 0, maxTop)

          el.scrollBy(dx, dy)

          // Let any post-scroll effects run, then re-assert the desired scroll position to avoid
          // capturing an intermediate state (Radix Select can re-sync scroll on open).
          await raf2()

          if (dx !== 0 && Math.abs(Number(el.scrollLeft ?? 0) - desiredLeft) > 1) {
            el.scrollLeft = desiredLeft
          }
          if (dy !== 0 && Math.abs(Number(el.scrollTop ?? 0) - desiredTop) > 1) {
            el.scrollTop = desiredTop
          }

          await raf2()
          return true
        },
        { selector: step.selector, dx: step.dx, dy: step.dy }
      )
      if (!ok) {
        throw new Error(`steps failed for ${name}: selector not found: ${step.selector}`)
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    if (step.action === "scrollTo") {
      const ok = await page.evaluate(
        async ({ selector, left, top }) => {
          const raf2 = () =>
            new Promise<void>((r) =>
              requestAnimationFrame(() => requestAnimationFrame(() => r()))
            )

          const els = Array.from(document.querySelectorAll(selector)).filter(
            (e): e is Element => e instanceof Element
          )
          if (els.length === 0) return false

          const scrollables = els.filter((e) => typeof (e as any).scrollTo === "function")
          if (scrollables.length === 0) return false

          const clamp = (v: number, min: number, max: number) =>
            Math.max(min, Math.min(max, v))

          let best: Element = scrollables[0]
          let bestScore = -1
          for (const el of scrollables) {
            const anyEl = el as any
            const scrollableX = Math.max(0, (anyEl.scrollWidth ?? 0) - (anyEl.clientWidth ?? 0))
            const scrollableY = Math.max(0, (anyEl.scrollHeight ?? 0) - (anyEl.clientHeight ?? 0))
            const score = scrollableX + scrollableY
            if (score > bestScore) {
              bestScore = score
              best = el
            }
          }

          const el = best as any

          // Wait for the scroll range to reach (or stabilize below) the requested position.
          let lastMaxTop = -1
          let stableFrames = 0
          for (let i = 0; i < 32; i++) {
            const maxTop = Math.max(
              0,
              (Number(el.scrollHeight ?? 0) || 0) - (Number(el.clientHeight ?? 0) || 0)
            )
            if (maxTop >= top) break
            if (maxTop === lastMaxTop) {
              stableFrames++
              if (stableFrames >= 2) break
            } else {
              stableFrames = 0
              lastMaxTop = maxTop
            }
            await raf2()
          }

          const maxLeft = Math.max(
            0,
            (Number(el.scrollWidth ?? 0) || 0) - (Number(el.clientWidth ?? 0) || 0)
          )
          const maxTop = Math.max(
            0,
            (Number(el.scrollHeight ?? 0) || 0) - (Number(el.clientHeight ?? 0) || 0)
          )
          const desiredLeft = clamp(left, 0, maxLeft)
          const desiredTop = clamp(top, 0, maxTop)

          el.scrollTo(desiredLeft, desiredTop)
          await raf2()

          if (Math.abs(Number(el.scrollLeft ?? 0) - desiredLeft) > 1) {
            el.scrollLeft = desiredLeft
          }
          if (Math.abs(Number(el.scrollTop ?? 0) - desiredTop) > 1) {
            el.scrollTop = desiredTop
          }

          await raf2()
          return true
        },
        { selector: step.selector, left: step.left, top: step.top }
      )
      if (!ok) {
        throw new Error(`steps failed for ${name}: selector not found: ${step.selector}`)
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    if (step.action === "type") {
      if (debug) {
        console.log(
          `- steps: ${name} step[${idx}] type ${step.selector} (${step.text.length} chars)`
        )
      }
      const expr = `(() => {
        const sel = ${JSON.stringify(step.selector)};
        const text = ${JSON.stringify(step.text)};
        const el = document.querySelector(sel);
        if (!el) return false;
        if (!(el instanceof HTMLInputElement) && !(el instanceof HTMLTextAreaElement)) return false;
        el.focus();
        // React-controlled inputs require using the native setter to avoid stale value tracking.
        const proto = el instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
        const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
        if (typeof setter !== "function") return false;
        setter.call(el, text);
        el.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
        el.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
        return true;
      })()`
      const ok = (await page.evaluate(expr)) as boolean
      if (!ok) {
        throw new Error(`steps failed for ${name}: selector not found: ${step.selector}`)
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    if (step.action === "attr") {
      const expr = `(() => {
        const sel = ${JSON.stringify(step.selector)};
        const name = ${JSON.stringify(step.name)};
        const value = ${JSON.stringify(step.value)};
        const el = document.querySelector(sel);
        if (!el || !(el instanceof Element)) return false;
        el.setAttribute(name, value);
        return true;
      })()`
      const ok = (await page.evaluate(expr)) as boolean
      if (!ok) {
        throw new Error(`steps failed for ${name}: selector not found: ${step.selector}`)
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    if (debug) {
      console.log(`- steps: ${name} step[${idx}] ${step.action} ${step.selector}`)
    }

    const point = await (async () => {
      const expr = `(() => {
        const rootSel = ${JSON.stringify(rootSel)};
        const sel = ${JSON.stringify(step.selector)};

        const root = document.querySelector(rootSel) || document.body;
        const el = document.querySelector(sel);
        if (!el || !(el instanceof Element)) return null;

        el.scrollIntoView({ block: "center", inline: "center" });
        const r = el.getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
      })()`
      return (await page.evaluate(expr)) as { x: number; y: number } | null
    })()

    if (!point) {
      throw new Error(`steps failed for ${name}: selector not found: ${step.selector}`)
    }

    if (step.action === "hover") {
      await page.mouse.move(point.x, point.y, { steps: 4 })
    } else if (step.action === "contextmenu") {
      await page.mouse.click(point.x, point.y, { button: "right", delay: 10 })
    } else if (step.action === "mouseDown") {
      await page.mouse.move(point.x, point.y, { steps: 4 })
      await page.mouse.down({ button: "left" })
    } else if (step.action === "mouseUp") {
      await page.mouse.move(point.x, point.y, { steps: 4 })
      await page.mouse.up({ button: "left" })
    } else if (step.action === "keys") {
      await page.focus(step.selector)
      for (const chord of step.keys) {
        for (const mod of chord.modifiers) {
          await page.keyboard.down(mod)
        }
        await page.keyboard.press(chord.key)
        for (const mod of [...chord.modifiers].reverse()) {
          await page.keyboard.up(mod)
        }
      }
    } else {
      await page.mouse.click(point.x, point.y, { button: "left", delay: 10 })
    }

    await waitForFonts(page, Math.min(2000, timeoutMs))
  }
}

async function applyOpenSteps(
  page: puppeteer.Page,
  name: string,
  timeoutMs: number,
  steps: OpenStep[],
  rootSel: string
) {
  const debug = process.env.DEBUG_GOLDENS === "1"

  for (const [idx, step] of steps.entries()) {
    if (step.action === "wait") {
      if (debug) console.log(`- openSteps: ${name} step[${idx}] wait ${step.waitMs}ms`)
      await new Promise((r) => setTimeout(r, step.waitMs))
      continue
    }

    if (step.action === "waitFor") {
      if (debug) console.log(`- openSteps: ${name} step[${idx}] waitFor ${step.selector}`)
      await page.waitForSelector(step.selector, { timeout: Math.min(timeoutMs, 30000) })
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    if (step.action === "move") {
      if (debug) console.log(`- openSteps: ${name} step[${idx}] move ${step.x},${step.y}`)
      await page.mouse.move(step.x, step.y, { steps: 4 })
      continue
    }
    if (step.action === "tabTo") {
      if (debug) {
        console.log(
          `- openSteps: ${name} step[${idx}] tabTo ${step.selector} (maxTabs=${step.maxTabs})`
        )
      }
      await page.evaluate(() => {
        const active = document.activeElement
        if (active instanceof HTMLElement) active.blur()
      })
      for (let attempt = 0; attempt < step.maxTabs; attempt++) {
        const ok = (await page.evaluate((selector) => {
          const el = document.activeElement
          return el instanceof Element && el.matches(selector)
        }, step.selector)) as boolean
        if (ok) break
        await page.keyboard.press("Tab")
      }
      const ok = (await page.evaluate((selector) => {
        const el = document.activeElement
        return el instanceof Element && el.matches(selector)
      }, step.selector)) as boolean
      if (!ok) {
        throw new Error(
          `openSteps failed for ${name}: tabTo did not reach ${step.selector} within ${step.maxTabs} tabs`
        )
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }
    if (step.action === "scroll") {
      const expr = `(() => {
        const sel = ${JSON.stringify(step.selector)};
        const dx = ${JSON.stringify(step.dx)};
        const dy = ${JSON.stringify(step.dy)};
        const el = document.querySelector(sel);
        if (!el || !(el instanceof Element)) return false;
        if (typeof (el).scrollBy !== "function") return false;
        (el).scrollBy(dx, dy);
        return true;
      })()`
      const ok = (await page.evaluate(expr)) as boolean
      if (!ok) {
        throw new Error(`openSteps failed for ${name}: selector not found: ${step.selector}`)
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    if (step.action === "type") {
      if (debug) {
        console.log(
          `- openSteps: ${name} step[${idx}] type ${step.selector} (${step.text.length} chars)`
        )
      }
      const expr = `(() => {
        const sel = ${JSON.stringify(step.selector)};
        const text = ${JSON.stringify(step.text)};
        const el = document.querySelector(sel);
        if (!el) return false;
        if (!(el instanceof HTMLInputElement) && !(el instanceof HTMLTextAreaElement)) return false;
        el.focus();
        // React-controlled inputs require using the native setter to avoid stale value tracking.
        const proto = el instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
        const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
        if (typeof setter !== "function") return false;
        setter.call(el, text);
        el.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
        el.dispatchEvent(new Event("change", { bubbles: true, composed: true }));
        return true;
      })()`
      const ok = (await page.evaluate(expr)) as boolean
      if (!ok) {
        throw new Error(`openSteps failed for ${name}: selector not found: ${step.selector}`)
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    if (step.action === "attr") {
      const expr = `(() => {
        const sel = ${JSON.stringify(step.selector)};
        const name = ${JSON.stringify(step.name)};
        const value = ${JSON.stringify(step.value)};
        const el = document.querySelector(sel);
        if (!el || !(el instanceof Element)) return false;
        el.setAttribute(name, value);
        return true;
      })()`
      const ok = (await page.evaluate(expr)) as boolean
      if (!ok) {
        throw new Error(`openSteps failed for ${name}: selector not found: ${step.selector}`)
      }
      await waitForFonts(page, Math.min(2000, timeoutMs))
      continue
    }

    const baseline = await openOverlayOutsideCounts(page, rootSel)

    if (debug) {
      console.log(
        `- openSteps: ${name} step[${idx}] ${step.action} ${step.selector} (baseline popper=${baseline.popperWrapper}, menu=${baseline.roleMenu})`
      )
    }

    const point = await (async () => {
      const expr = `(() => {
        const rootSel = ${JSON.stringify(rootSel)};
        const sel = ${JSON.stringify(step.selector)};

        const root = document.querySelector(rootSel) || document.body;
        const el = document.querySelector(sel);
        if (!el || !(el instanceof Element)) return null;

        el.scrollIntoView({ block: "center", inline: "center" });
        const r = el.getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
      })()`
      return (await page.evaluate(expr)) as { x: number; y: number } | null
    })()

    if (!point) {
      throw new Error(`openSteps failed for ${name}: selector not found: ${step.selector}`)
    }

    if (step.action === "hover") {
      await page.mouse.move(point.x, point.y, { steps: 4 })
    } else if (step.action === "contextmenu") {
      await page.mouse.click(point.x, point.y, { button: "right", delay: 10 })
    } else if (step.action === "mouseDown") {
      await page.mouse.move(point.x, point.y, { steps: 4 })
      await page.mouse.down({ button: "left" })
    } else if (step.action === "mouseUp") {
      await page.mouse.move(point.x, point.y, { steps: 4 })
      await page.mouse.up({ button: "left" })
    } else if (step.action === "keys") {
      await page.focus(step.selector)
      for (const chord of step.keys) {
        for (const mod of chord.modifiers) {
          await page.keyboard.down(mod)
        }
        await page.keyboard.press(chord.key)
        for (const mod of [...chord.modifiers].reverse()) {
          await page.keyboard.up(mod)
        }
      }
    } else {
      await page.mouse.click(point.x, point.y, { button: "left", delay: 10 })
    }

    // Wait for a new portal-ish surface to appear (submenu, nested menu, etc.).
    const waitExpr = `(() => {
      const root = document.querySelector(${JSON.stringify(rootSel)}) || document.body;
      const outsideCount = (sel) =>
        Array.from(document.querySelectorAll(sel)).filter((el) => !root.contains(el)).length;

      const popperWrapper = outsideCount("[data-radix-popper-content-wrapper]");
      const roleMenu = outsideCount("[role='menu']");
      const roleDialog = outsideCount("[role='dialog']");
      const dataStateOpen = outsideCount("[data-state='open']");
      const dataSonnerToast = outsideCount("[data-sonner-toast]");

      return (
        popperWrapper > ${baseline.popperWrapper} ||
        roleMenu > ${baseline.roleMenu} ||
        roleDialog > ${baseline.roleDialog} ||
        dataStateOpen > ${baseline.dataStateOpen} ||
        dataSonnerToast > ${baseline.dataSonnerToast}
      );
    })()`

    await waitForExpr(page, waitExpr, Math.min(timeoutMs, 2500))
    await waitForFonts(page, Math.min(2000, timeoutMs))
  }
}

function inferOpenAction(name: string): OpenAction {
  if (name === "context-menu-demo") return "contextmenu"
  if (name === "tooltip-demo" || name === "hover-card-demo") return "hover"
  if (name === "command-dialog") return "keys"
  return "click"
}

function inferOpenKeys(name: string): KeyChord | null {
  // `command-dialog` toggles on keydown: (metaKey || ctrlKey) + "j".
  if (name === "command-dialog") return parseOpenKeys("Control+KeyJ")
  return null
}

async function disableMotion(page: puppeteer.Page) {
  const expr = `(() => {
    const id = "fret-disable-motion";
    if (document.getElementById(id)) return true;

    const style = document.createElement("style");
    style.id = id;
    style.textContent = [
      "* {",
      "  animation: none !important;",
      "  transition: none !important;",
      "  scroll-behavior: auto !important;",
      "}",
      "*::before, *::after {",
      "  animation: none !important;",
      "  transition: none !important;",
      "}",
    ].join("\\n");

    document.head.appendChild(style);
    return true;
  })()`

  await page.evaluate(expr)
}

async function waitForExpr(
  page: puppeteer.Page,
  expr: string,
  timeoutMs: number,
  intervalMs = 50
) {
  const debug = process.env.DEBUG_GOLDENS === "1"
  if (debug) {
    console.log(`- waitForExpr: enter (timeoutMs=${timeoutMs})`)
  }
  const deadline = Date.now() + timeoutMs
  let tries = 0
  while (Date.now() < deadline) {
    tries++
    // Same `__name(...)` caveat as `extractOne`: pass an expression string instead of a function.
    const evalBudget = Math.max(1, Math.min(2000, deadline - Date.now()))
    let ok = false
    try {
      ok = (await Promise.race([
        page.evaluate(expr) as Promise<boolean>,
        new Promise<boolean>((_, reject) =>
          setTimeout(() => reject(new Error("eval timeout")), evalBudget)
        ),
      ])) as boolean
    } catch (error) {
      if (debug && tries === 1) {
        console.log(`- waitForExpr: first eval failed: ${String(error)}`)
      }
    }

    if (ok) return
    await new Promise((r) => setTimeout(r, intervalMs))
  }
  throw new Error(
    `timeout waiting for page expression after ${timeoutMs}ms (tries=${tries})`
  )
}

async function waitForFonts(page: puppeteer.Page, timeoutMs: number) {
  try {
    await page.evaluate(`(async () => {
      if (document.fonts && document.fonts.status !== "loaded") {
        await Promise.race([
          document.fonts.ready,
          new Promise((r) => setTimeout(r, ${timeoutMs})),
        ]);
      }
    })()`)
  } catch {
    // ignore
  }
}

async function ensureGoldenTarget(page: puppeteer.Page) {
  const expr = `(() => {
    const existing = document.querySelector("[data-fret-golden-target]");
    if (existing) return true;

    // The /view/[style]/[name] route wraps the rendered registry component in a bg-background div.
    // If upstream changes and this selector breaks, regenerate goldens after updating this heuristic.
    const root = document.querySelector(".bg-background");
    if (!root) return false;

    root.setAttribute("data-fret-golden-root", "true");

    const wrapper = document.createElement("div");
    wrapper.setAttribute("data-fret-golden-target", "true");

    // Move *all* child nodes (including text nodes) into the wrapper.
    while (root.firstChild) {
      wrapper.appendChild(root.firstChild);
    }
    root.appendChild(wrapper);
    return true;
  })()`

  const ok = await page.evaluate(expr)
  if (!ok) {
    throw new Error(
      "failed to locate shadcn view wrapper (.bg-background) to attach data-fret-golden-target"
    )
  }
}

async function injectCssLinks(page: puppeteer.Page, urls: string[]) {
  if (urls.length === 0) return

  await page.evaluate((urls) => {
    const existing = new Set(
      Array.from(document.querySelectorAll("link[rel=stylesheet]"))
        .map((l) => (l as HTMLLinkElement).href)
        .filter(Boolean)
    )

    for (const url of urls) {
      if (existing.has(url)) continue
      const link = document.createElement("link")
      link.rel = "stylesheet"
      link.href = url
      document.head.appendChild(link)
    }
  }, urls)
}

async function waitForShadcnStyles(page: puppeteer.Page, timeoutMs: number) {
  const debug = process.env.DEBUG_GOLDENS === "1"
  if (debug) {
    console.log(`- waitForShadcnStyles: enter (timeoutMs=${timeoutMs})`)
  }
  // Same `__name(...)` caveat as `extractOne`: pass an expression string instead of a function.
  //
  // We intentionally use a *global* Tailwind sentinel (body classes from `app/layout.tsx`) instead
  // of component-specific heuristics (e.g. `rounded-none` would legitimately produce `0px` radius).
  const expr = `(() => {
    const root = document.querySelector("[data-fret-golden-target]");
    if (!root) return false;
    const body = document.body;
    if (!body) return false;
    const cs = window.getComputedStyle(body);
    // overscroll-none + antialiased are always present on <body> in this app.
    const overscrollOk =
      cs.overscrollBehaviorX === "none" || cs.overscrollBehaviorY === "none";
    const smoothOk =
      cs.webkitFontSmoothing === "antialiased" ||
      cs.MozOsxFontSmoothing === "grayscale";
    return overscrollOk || smoothOk;
  })()`

  await waitForExpr(page, expr, timeoutMs)
}

async function settleRaf(page: puppeteer.Page, frames = 2) {
  const n = Math.max(1, Math.min(8, Math.floor(frames)))
  const expr = `(() => new Promise((resolve) => {
    let left = ${n};
    const tick = () => {
      left -= 1;
      if (left <= 0) return resolve(true);
      requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  }))()`
  await page.evaluate(expr)
}

async function waitForRechartsSeriesIfPresent(
  page: puppeteer.Page,
  timeoutMs: number
) {
  const debug = process.env.DEBUG_GOLDENS === "1"
  if (debug) {
    console.log(`- waitForRechartsSeriesIfPresent: enter (timeoutMs=${timeoutMs})`)
  }

  const expr = `(() => {
    const root = document.querySelector("[data-fret-golden-target]") || document.body;
    if (!root) return false;

    const wrapper =
      root.querySelector(".recharts-wrapper") ||
      root.querySelector("svg.recharts-surface");
    const hasRecharts = !!wrapper;
    if (!hasRecharts) return true;

    const wrapperRect =
      wrapper && wrapper instanceof Element ? wrapper.getBoundingClientRect() : null;
    const minDim =
      wrapperRect && wrapperRect.width > 0 && wrapperRect.height > 0
        ? Math.min(wrapperRect.width, wrapperRect.height)
        : 0;
    const minSeriesDim = minDim > 0 ? Math.max(10, minDim * 0.2) : 10;

    const bestRect = (nodes) => {
      let best = null;
      let bestArea = 0;
      for (const el of Array.from(nodes)) {
        if (!(el instanceof Element)) continue;
        const r = el.getBoundingClientRect();
        const area = r.width * r.height;
        if (area > bestArea) {
          bestArea = area;
          best = { x: r.x, y: r.y, w: r.width, h: r.height };
        }
      }
      return best;
    };

    const stableRect = (rect) => {
      const w = rect.w || 0;
      const h = rect.h || 0;
      if (w <= 0 || h <= 0) return false;
      if (w < minSeriesDim && h < minSeriesDim) return false;

      const now = performance.now();
      const key = "__fretRechartsStable";
      // eslint-disable-next-line no-undef
      const state = (window[key] ||= {
        x: rect.x,
        y: rect.y,
        w,
        h,
        since: now,
      });

      const eps = 0.5;
      const changed =
        Math.abs(state.x - rect.x) > eps ||
        Math.abs(state.y - rect.y) > eps ||
        Math.abs(state.w - w) > eps ||
        Math.abs(state.h - h) > eps;

      if (changed) {
        state.x = rect.x;
        state.y = rect.y;
        state.w = w;
        state.h = h;
        state.since = now;
        return false;
      }

      return now - state.since >= 200;
    };

    // Prefer per-chart-type series layers to avoid picking up axis/grid shapes.
    const radarLayer = root.querySelector("g.recharts-layer.recharts-radar");
    if (radarLayer) {
      const rect = bestRect(radarLayer.querySelectorAll("path,polygon,rect,circle"));
      return rect ? stableRect(rect) : false;
    }

    const radialLayer = root.querySelector("g.recharts-layer.recharts-radial-bar");
    if (radialLayer) {
      const rect = bestRect(radialLayer.querySelectorAll("path,polygon,rect,circle"));
      return rect ? stableRect(rect) : false;
    }

    const pieLayer = root.querySelector("g.recharts-layer.recharts-pie");
    if (pieLayer) {
      const rect = bestRect(pieLayer.querySelectorAll("path,polygon,rect,circle"));
      return rect ? stableRect(rect) : false;
    }

    const barLayer = root.querySelector("g.recharts-layer.recharts-bar-rectangles");
    if (barLayer) {
      const rect = bestRect(barLayer.querySelectorAll("path,rect"));
      return rect ? stableRect(rect) : false;
    }

    const areaLayer = root.querySelector("g.recharts-layer.recharts-area");
    if (areaLayer) {
      const rect = bestRect(areaLayer.querySelectorAll("path"));
      return rect ? stableRect(rect) : false;
    }

    const lineLayer = root.querySelector("g.recharts-layer.recharts-line");
    if (lineLayer) {
      const rect = bestRect(lineLayer.querySelectorAll("path"));
      return rect ? stableRect(rect) : false;
    }

    // Fallback: any known series-ish node with non-trivial bounds.
    const candidates = root.querySelectorAll(
      ".recharts-curve,.recharts-rectangle,.recharts-sector,.recharts-radial-bar-sector"
    );
    const rect = bestRect(candidates);
    return rect ? stableRect(rect) : false;
  })()`

  await waitForExpr(page, expr, timeoutMs, 25)
}

async function setThemeBeforeLoad(page: puppeteer.Page, theme: Theme) {
  await page.evaluateOnNewDocument((theme) => {
    localStorage.setItem("theme", theme)
  }, theme)
}

function parseFreezeDate(iso: string): { year: number; month: number; day: number } {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso.trim())
  if (!m) {
    throw new Error(
      `invalid --freezeDate=${iso} (expected YYYY-MM-DD)`
    )
  }

  const year = Number(m[1])
  const month = Number(m[2])
  const day = Number(m[3])
  if (
    !Number.isFinite(year) ||
    !Number.isFinite(month) ||
    !Number.isFinite(day) ||
    month < 1 ||
    month > 12 ||
    day < 1 ||
    day > 31
  ) {
    throw new Error(
      `invalid --freezeDate=${iso} (expected YYYY-MM-DD)`
    )
  }

  return { year, month, day }
}

async function freezeDateBeforeLoad(page: puppeteer.Page, freezeDate: string) {
  const { year, month, day } = parseFreezeDate(freezeDate)

  // Keep date/time formatting deterministic across machines.
  // Puppeteer throws if the timezone ID is unknown, so keep it best-effort.
  try {
    await (page as any).emulateTimezone?.("UTC")
  } catch {
    // ignore
  }

  // Use noon UTC to avoid DST / midnight edge cases.
  const ts = Date.UTC(year, month - 1, day, 12, 0, 0)

  await page.evaluateOnNewDocument((ts) => {
    const OriginalDate = Date
    class FrozenDate extends OriginalDate {
      constructor(...args: any[]) {
        if (args.length === 0) {
          super(ts)
          return
        }
        // @ts-ignore - Date constructor overloads are not represented precisely.
        super(...args)
      }
      static now() {
        return ts
      }
    }

    // Preserve static helpers.
    ;(FrozenDate as any).UTC = (OriginalDate as any).UTC
    ;(FrozenDate as any).parse = (OriginalDate as any).parse
    ;(FrozenDate as any).prototype = (OriginalDate as any).prototype

    // @ts-ignore - we intentionally override the global Date constructor.
    globalThis.Date = FrozenDate
  }, ts)
}

function repoRootFromScript(): string {
  const envRoot = process.env.REPO_ROOT
  if (envRoot) {
    return path.resolve(envRoot)
  }

  const scriptPath = fileURLToPath(import.meta.url)
  const scriptDir = path.dirname(scriptPath)
  return path.resolve(scriptDir, "../../..")
}

const repoRoot = repoRootFromScript()

async function loadPuppeteer(): Promise<typeof import("puppeteer")> {
  const require = createRequire(import.meta.url)

  const candidates = [
    path.join(repoRoot, "repo-ref", "ui"),
    repoRoot,
    process.cwd(),
  ]

  for (const candidate of candidates) {
    try {
      const entry = require.resolve("puppeteer", { paths: [candidate] })
      const mod = await import(pathToFileURL(entry).href)
      return ((mod as any).default ?? mod) as typeof import("puppeteer")
    } catch {
      // keep searching
    }
  }

  // Fall back to normal Node resolution (e.g. if the caller installed puppeteer globally or in CWD).
  const mod = await import("puppeteer")
  return ((mod as any).default ?? mod) as typeof import("puppeteer")
}

async function resolveCssInjectionUrls(style: string, baseUrl: string) {
  // Next's HTML output for this route can omit some CSS links depending on how RSC streaming resolves.
  // We use the server-side RSC manifest to discover the actual CSS chunks and inject them ourselves.
  const manifestPath = path.join(
    repoRoot,
    "repo-ref",
    "ui",
    "apps",
    "v4",
    ".next",
    "server",
    "app",
    "(view)",
    "view",
    "[style]",
    "[name]",
    "page_client-reference-manifest.js"
  )

  if (!fs.existsSync(manifestPath)) {
    return []
  }

  ;(globalThis as any).__RSC_MANIFEST = {}
  await import(pathToFileURL(manifestPath).href)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const entry = (globalThis as any).__RSC_MANIFEST?.[
    "/(view)/view/[style]/[name]/page"
  ]
  if (!entry) {
    throw new Error("Failed to read __RSC_MANIFEST entry for /view/[style]/[name]")
  }

  const cssFiles: string[] = []
  const add = (p: string) => {
    const url = `${baseUrl}/_next/${p.replace(/^[\\/]+/, "")}`
    cssFiles.push(url)
  }

  const entryCssFiles = entry.entryCSSFiles as Record<string, unknown> | undefined
  if (entryCssFiles) {
    for (const value of Object.values(entryCssFiles)) {
      if (!Array.isArray(value)) continue
      for (const item of value) {
        if (typeof (item as any)?.path === "string") {
          add((item as any).path)
        }
      }
    }
  }

  // Also inject the style-specific theme file if present (legacyStyles currently only includes
  // `new-york-v4`, but we keep this generic).
  const styleManifestPath = path.join(
    repoRoot,
    "repo-ref",
    "ui",
    "apps",
    "v4",
    "public",
    "r",
    "styles",
    style,
    "index.json"
  )
  if (fs.existsSync(styleManifestPath)) {
    // Some styles register extra CSS under public/r/styles/<style>/...
    // We don't hardcode those here; style-specific coverage can be added later if needed.
  }

  return Array.from(new Set(cssFiles))
}

async function run(options: GoldenOptions): Promise<string[]> {
  ensureDir(options.outDir)
  const debug = process.env.DEBUG_GOLDENS === "1"

  // Fail fast if the server isn't reachable.
  {
    const testName = options.names[0]
    const url = `${options.baseUrl}/view/${options.style}/${testName}`
    const deadline = Date.now() + Math.min(5000, options.timeoutMs)
    let lastError: unknown = undefined
    let lastStatus: number | undefined = undefined
    let lastHtml: string | undefined = undefined

    try {
      while (Date.now() < deadline) {
        const controller = new AbortController()
        const timeout = setTimeout(() => controller.abort(), 1500)
        try {
          const resp = await fetch(url, { signal: controller.signal })
          lastStatus = resp.status
          if (resp.ok) {
            lastHtml = await resp.text()
            lastError = undefined
            break
          }
          lastError = new Error(`HTTP ${resp.status} ${resp.statusText}`)
        } catch (error) {
          lastError = error
        } finally {
          clearTimeout(timeout)
        }

        await new Promise((r) => setTimeout(r, 250))
      }

      if (lastError) {
        throw lastError
      }
    } catch (error) {
      console.error(`! Cannot load shadcn view page (status: ${lastStatus ?? "n/a"})`)
      console.error(`  Server: ${options.baseUrl}`)
      console.error(`  Then open: ${url}`)
      throw error
    }

    // Golden extraction should be run against a production server (`next start`), otherwise
    // turbopack dev output may not expose stable CSS/JS chunk URLs (and computed styles will be wrong).
    if (
      lastHtml &&
      (lastHtml.includes("hmr-client") ||
        lastHtml.includes("next-devtools") ||
        lastHtml.includes("turbopack_browser_dev_hmr-client"))
    ) {
      console.error("! Detected a Next.js dev server response.")
      console.error("  Golden extraction requires production assets (CSS chunks) to be reachable.")
      console.error("  Recommended flow:")
      console.error("    1) pnpm -C repo-ref/ui --filter shadcn build")
      console.error(
        "    2) NEXT_PUBLIC_APP_URL=http://localhost:4020 pnpm -C repo-ref/ui/apps/v4 exec next build --webpack"
      )
      console.error(
        "    3) NEXT_PUBLIC_APP_URL=http://localhost:4020 pnpm -C repo-ref/ui/apps/v4 exec next start -p 4020"
      )
      console.error(
        `  Then rerun with:\n    --baseUrl=http://localhost:4020`
      )
      throw new Error("refusing to run against dev server (use next start)")
    }
  }

  const executablePath = resolveBrowserExecutablePath()

  const puppeteer = await loadPuppeteer()

  let browser: puppeteer.Browser
  try {
    browser = await puppeteer.launch({
      ...(executablePath ? { executablePath } : {}),
      headless: "new",
      protocolTimeout: Math.max(180_000, options.timeoutMs + 30_000),
      defaultViewport: {
        width: options.viewportW,
        height: options.viewportH,
        deviceScaleFactor: options.deviceScaleFactor,
      },
    })
  } catch (error) {
    console.error("! Failed to launch a browser for puppeteer.")
    console.error(
      "  If you don't have Chrome/Edge installed, install puppeteer's browser:"
    )
    console.error("  pnpm -C repo-ref/ui dlx puppeteer browsers install chrome")
    console.error(
      "  Or set PUPPETEER_EXECUTABLE_PATH to your local browser executable."
    )
    throw error
  }

  try {
    const failures: string[] = []

    const cssInjectionUrls = await resolveCssInjectionUrls(
      options.style,
      options.baseUrl
    )
    console.log(`- injectCss: ${cssInjectionUrls.length} stylesheets`)

    // Sanity check: ensure Tailwind utilities are actually applied (otherwise computed styles
    // look like browser defaults and the goldens are not useful).
    //
    // We only check `button-default` because it is a stable, low-dependency example.
    if (options.names.includes("button-default")) {
      const page = await browser.newPage()
      page.setDefaultTimeout(options.timeoutMs)
      page.setDefaultNavigationTimeout(options.timeoutMs)
      await page.emulateMediaFeatures([
        { name: "prefers-reduced-motion", value: "reduce" },
      ])
      if (options.freezeDate) {
        await freezeDateBeforeLoad(page, options.freezeDate)
      }
      await setThemeBeforeLoad(page, options.themes[0] ?? "light")
      const url = `${options.baseUrl}/view/${options.style}/button-default`
      await page.goto(url, { waitUntil: "networkidle2", timeout: options.timeoutMs })
      await page.waitForSelector("body", { timeout: 30000 })
      await ensureGoldenTarget(page)
      await page.waitForSelector("[data-fret-golden-target]", { timeout: 30000 })
      await injectCssLinks(page, cssInjectionUrls)
      await waitForShadcnStyles(page, Math.min(30000, options.timeoutMs))
      const ok = await page.evaluate(`(() => {
        const root = document.querySelector("[data-fret-golden-target]");
        if (!root) return false;
        const button = root.querySelector("button");
        if (!button) return false;
        const cs = window.getComputedStyle(button);
        return cs.display === "inline-flex" && cs.borderTopWidth === "0px";
      })()`)
      await page.close()
      if (!ok) {
        throw new Error(
          "Tailwind utilities do not appear to be applied (button-default still looks like browser defaults)."
        )
      }
    }

    const pagesByTheme: Record<string, puppeteer.Page> = {}
    for (const theme of options.themes) {
      const page = await browser.newPage()
      page.setDefaultTimeout(options.timeoutMs)
      page.setDefaultNavigationTimeout(options.timeoutMs)
      await page.emulateMediaFeatures([
        { name: "prefers-reduced-motion", value: "reduce" },
      ])
      if (options.freezeDate) {
        await freezeDateBeforeLoad(page, options.freezeDate)
      }
      await setThemeBeforeLoad(page, theme)
      pagesByTheme[theme] = page
    }

    for (const name of options.names) {
      for (const mode of options.modes) {
        const variants: Array<GoldenVariant | OpenVariant | null> =
          options.variants && options.variants.length > 0
            ? options.variants
            : mode === "open" && options.openVariants && options.openVariants.length > 0
              ? options.openVariants
              : [null]

        for (const variant of variants) {
          const variantSuffix = variant ? `.${variant.variant}` : ""
          const suffix = mode === "closed" ? `${variantSuffix}` : `${variantSuffix}.${mode}`
          const outPath = path.join(options.outDir, `${name}${suffix}.json`)
          if (!options.update && fs.existsSync(outPath)) {
            continue
          }

          const out: GoldenFile = (() => {
            if (options.mergeThemes && fs.existsSync(outPath)) {
              const existing = JSON.parse(fs.readFileSync(outPath, "utf8")) as GoldenFile
              const existingMode = existing.mode ?? "closed"
              if (
                existing.version !== 1 ||
                existing.style !== options.style ||
                existing.name !== name ||
                existingMode !== mode
              ) {
                throw new Error(
                  `refusing to merge themes into ${outPath} (mismatched header)`
                )
              }
              return existing
            }

            return {
              version: 1,
              style: options.style,
              name,
              mode,
              variant: variant?.variant,
              themes: {},
            }
          })()

          let ok = true
          for (const theme of options.themes) {
            const url = `${options.baseUrl}/view/${options.style}/${name}`
            try {
              const page = pagesByTheme[theme]

              if (debug) console.log(`- goto: ${name}${suffix} (${theme})`)
              await page.goto(url, {
                waitUntil: "networkidle2",
                timeout: options.timeoutMs,
              })
              // Puppeteer mouse/keyboard input targets the active tab. When we keep one page per
              // theme, ensure we always bring the current page to front before running steps.
              await page.bringToFront()
              await page.waitForSelector("body", { timeout: 30000 })
              if (debug) {
                console.log(`- ensureGoldenTarget: ${name}${suffix} (${theme})`)
              }
              await ensureGoldenTarget(page)
              await page.waitForSelector("[data-fret-golden-target]", { timeout: 30000 })
              if (debug) console.log(`- injectCssLinks: ${name}${suffix} (${theme})`)
              await injectCssLinks(page, cssInjectionUrls)

              // Ensure stable geometry: shadcn overlays use enter/exit animations that can affect
              // `getBoundingClientRect()` if captured mid-transition.
              await disableMotion(page)

              await page.evaluate(`(() => {
                const indicator = document.querySelector("[data-tailwind-indicator]");
                if (indicator) indicator.remove();
              })()`)

              await waitForFonts(page, Math.min(2000, options.timeoutMs))
              if (debug) console.log(`- waitForShadcnStyles: ${name}${suffix} (${theme})`)
              await waitForShadcnStyles(page, Math.min(30000, options.timeoutMs))

              let openMeta: OpenMeta | null = null
              if (mode === "open") {
                const action = options.openAction ?? inferOpenAction(name)
                const selector =
                  (variant && "selector" in variant ? variant.selector : undefined) ??
                  options.openSelector
                const keys =
                  action === "keys"
                    ? options.openKeys ?? inferOpenKeys(name) ?? undefined
                    : undefined
                openMeta = await openOverlay(
                  page,
                  name,
                  options.timeoutMs,
                  selector,
                  action,
                  keys
                )

                if (options.openSteps && options.openSteps.length > 0) {
                  await applyOpenSteps(
                    page,
                    name,
                    options.timeoutMs,
                    options.openSteps,
                    "[data-fret-golden-target]"
                  )
                }
              }

              if (options.steps && options.steps.length > 0) {
                await applySteps(
                  page,
                  name,
                  options.timeoutMs,
                  options.steps,
                  "[data-fret-golden-target]"
                )
              }

              // Some pages (notably Recharts-backed `chart-*` examples) render key SVG nodes
              // asynchronously (ResizeObserver + RAF + JS-driven animation). Wait for those
              // nodes to exist with non-trivial bounds to keep goldens stable across themes.
              await settleRaf(page, 2)
              await waitForRechartsSeriesIfPresent(
                page,
                Math.min(8000, options.timeoutMs)
              )
              await settleRaf(page, 2)

              if (debug) console.log(`- extractOne: ${name}${suffix} (${theme})`)
              const extracted = await extractOne(page)
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              ;(extracted as any).open = openMeta
              // Normalize a few floats that may slip through as high precision.
              // eslint-disable-next-line @typescript-eslint/no-explicit-any
              ;(extracted as any).devicePixelRatio = round3(
                // eslint-disable-next-line @typescript-eslint/no-explicit-any
                (extracted as any).devicePixelRatio
              )

              ;(out.themes as Record<string, unknown>)[theme] = extracted
            } catch (error) {
              ok = false
              const msg = `${name}${suffix} (${theme}): ${String(error)}`
              failures.push(msg)
              console.error(`! failed ${name}${suffix} (${theme})`)
              console.error(`  url: ${url}`)
              console.error(`  error: ${String(error)}`)

              // Try to recover by recreating the page for this theme so later iterations can continue.
              try {
                await pagesByTheme[theme].close()
              } catch {
                // ignore
              }
              const page = await browser.newPage()
              page.setDefaultTimeout(options.timeoutMs)
              await page.emulateMediaFeatures([
                { name: "prefers-reduced-motion", value: "reduce" },
              ])
              await setThemeBeforeLoad(page, theme as Theme)
              pagesByTheme[theme] = page
            }
          }

          if (ok) {
            writeIfChanged(outPath, out, options.update)
            console.log(`- wrote ${path.relative(repoRoot, outPath)}`)
          }
        }
      }
    }

    for (const theme of options.themes) {
      await pagesByTheme[theme].close()
    }

    return failures
  } finally {
    await browser.close()
  }
}

const { flags, names } = parseArgs(process.argv.slice(2))

const style =
  (typeof flags.style === "string" ? flags.style : undefined) ??
  process.env.STYLE ??
  "new-york-v4"

const baseUrlRaw =
  (typeof flags.baseUrl === "string" ? flags.baseUrl : undefined) ??
  process.env.BASE_URL ??
  "http://localhost:4000"

const startServer =
  flags.startServer === true || process.env.START_SERVER === "1"

const nextDir =
  (typeof flags.nextDir === "string" ? flags.nextDir : undefined) ??
  process.env.NEXT_DIR ??
  path.join(repoRoot, "repo-ref", "ui", "apps", "v4")

const typesRaw =
  (typeof flags.types === "string" ? flags.types : undefined) ??
  process.env.TYPES ??
  // Note: `/view/[style]/[name]` only pre-renders `registry:block|component|example|internal`.
  // `registry:ui` entries are leaf library sources and are not routable by default.
  "registry:block,registry:component,registry:example"
const types = typesRaw
  .split(",")
  .map((t) => t.trim())
  .filter(Boolean)

const outDir =
  (typeof flags.outDir === "string" ? flags.outDir : undefined) ??
  process.env.OUT_DIR ??
  path.join(repoRoot, "goldens", "shadcn-web", "v4", style)

const themesRaw =
  (typeof flags.themes === "string" ? flags.themes : undefined) ??
  process.env.THEMES ??
  "light,dark"

const themes = themesRaw
  .split(",")
  .map((t) => t.trim())
  .filter(Boolean)
  .filter((t): t is Theme => t === "light" || t === "dark")

const timeoutMs =
  Number(
    (typeof flags.timeoutMs === "string" ? flags.timeoutMs : undefined) ??
      process.env.TIMEOUT_MS ??
      "60000"
  ) || 60000

const viewportW =
  Number(
    (typeof flags.viewportW === "string" ? flags.viewportW : undefined) ??
      process.env.VIEWPORT_W ??
      "1440"
  ) || 1440

const viewportH =
  Number(
    (typeof flags.viewportH === "string" ? flags.viewportH : undefined) ??
      process.env.VIEWPORT_H ??
      "900"
  ) || 900

const deviceScaleFactor =
  Number(
    (typeof flags.deviceScaleFactor === "string"
      ? flags.deviceScaleFactor
      : undefined) ??
      process.env.DEVICE_SCALE_FACTOR ??
      "2"
  ) || 2

const update = flags.update === true || process.env.UPDATE_GOLDENS === "1"

const modesRaw =
  (typeof flags.modes === "string" ? flags.modes : undefined) ??
  process.env.MODES ??
  (flags.open === true ? "closed,open" : "closed")

const modes = modesRaw
  .split(",")
  .map((m) => m.trim())
  .filter(Boolean)
  .filter((m): m is Mode => m === "closed" || m === "open")

const openSelector =
  (typeof flags.openSelector === "string" ? flags.openSelector : undefined) ??
  process.env.OPEN_SELECTOR ??
  undefined

const openVariantsRaw =
  (typeof flags.openVariants === "string" ? flags.openVariants : undefined) ??
  process.env.OPEN_VARIANTS ??
  undefined

const openVariants = openVariantsRaw ? parseOpenVariants(openVariantsRaw) : undefined

const variantsRaw =
  (typeof flags.variants === "string" ? flags.variants : undefined) ??
  process.env.VARIANTS ??
  undefined

const variants = variantsRaw ? parseVariants(variantsRaw) : undefined

const openActionRaw =
  (typeof flags.openAction === "string" ? flags.openAction : undefined) ??
  process.env.OPEN_ACTION ??
  undefined

const openAction = (() => {
  if (!openActionRaw) return undefined
  const v = openActionRaw.trim()
  if (v === "click" || v === "hover" || v === "contextmenu" || v === "keys") {
    return v
  }
  throw new Error(
    `invalid --openAction=${openActionRaw} (expected click|hover|contextmenu|keys)`
  )
})()

const openKeysRaw =
  (typeof flags.openKeys === "string" ? flags.openKeys : undefined) ??
  process.env.OPEN_KEYS ??
  undefined

const openKeys = openKeysRaw ? parseOpenKeys(openKeysRaw) : undefined

const openStepsRaw =
  (typeof flags.openSteps === "string" ? flags.openSteps : undefined) ??
  process.env.OPEN_STEPS ??
  undefined

const openSteps = openStepsRaw ? parseOpenSteps(openStepsRaw, openKeys) : undefined

const stepsRaw =
  (typeof flags.steps === "string" ? flags.steps : undefined) ??
  process.env.STEPS ??
  undefined

const steps = stepsRaw ? parseOpenSteps(stepsRaw, openKeys) : undefined

const freezeDate =
  (typeof flags.freezeDate === "string" ? flags.freezeDate : undefined) ??
  process.env.FREEZE_DATE ??
  undefined

const defaultNames = ["button-default", "tabs-demo"]
const all = flags.all === true || process.env.ALL_GOLDENS === "1"

async function resolveNames(): Promise<string[]> {
  if (!all) {
    return names.length > 0 ? names : defaultNames
  }

  const indexPath = path.join(
    repoRoot,
    "repo-ref",
    "ui",
    "apps",
    "v4",
    "registry",
    "__index__.tsx"
  )
  if (!fs.existsSync(indexPath)) {
    throw new Error(
      `missing shadcn registry index at ${indexPath}; either provide explicit names or install repo-ref/ui`
    )
  }

  const mod = await import(
    pathToFileURL(
      indexPath
    ).href
  )
  const index = (mod as any).Index?.[style] as Record<string, any> | undefined
  if (!index) {
    throw new Error(`missing registry Index for style "${style}"`)
  }

  return Object.keys(index)
    .filter((name) => types.includes(index[name]?.type))
    .sort()
}

function printHelp() {
  console.log("shadcn web golden extract")
  console.log("")
  console.log("Common flags:")
  console.log("  --baseUrl=http://localhost:4020")
  console.log("  --style=new-york-v4")
  console.log("  --themes=light,dark")
  console.log("  --modes=closed,open  (or --open)")
  console.log("  --freezeDate=YYYY-MM-DD  (optional; makes time-dependent examples deterministic)")
  console.log("  --outDir=...")
  console.log("  --update")
  console.log("  --all")
  console.log("")
  console.log("Notes:")
  console.log("  - Positional arguments are *route names* (e.g. \"chart-line-interactive\").")
  console.log("  - Do not pass output-key suffixes like \".hover-mid\" or \".open\" as part of the name.")
  console.log("  - Use --variants=... and/or --modes=... instead.")
  console.log("")
  console.log("Viewport flags:")
  console.log("  --viewportW=1440 --viewportH=900 --deviceScaleFactor=2")
  console.log("")
  console.log("Open-mode flags:")
  console.log("  --openAction=click|hover|contextmenu|keys")
  console.log("  --openSelector=<css>")
  console.log("  --openVariants=\"<variant>=<css>;<variant>=<css>\"")
  console.log("  --openSteps=\"...\"")
  console.log("")
  console.log("Server flags:")
  console.log("  --startServer        Start a Next.js production server in-process")
  console.log("  --nextDir=<path>     Next.js app dir (default: repo-ref/ui/apps/v4)")
}

if (flags.help === true || flags.h === true) {
  printHelp()
  process.exit(0)
}

let startedServer: StartedServer | null = null
let baseUrl = baseUrlRaw

try {
  console.log(`?? shadcn web golden extract`)
  console.log(`- baseUrl: ${baseUrl}`)
  console.log(`- startServer: ${startServer ? "yes" : "no"}`)
  console.log(`- nextDir: ${nextDir}`)
  console.log(`- style: ${style}`)
  console.log(`- themes: ${themes.join(", ")}`)
  console.log(`- modes: ${modes.join(", ")}`)
  console.log(`- types: ${types.join(", ")}`)
  console.log(`- outDir: ${outDir}`)
  console.log(`- timeoutMs: ${timeoutMs}`)
  console.log(`- viewport: ${viewportW}x${viewportH} @${deviceScaleFactor}x`)
  console.log(`- update: ${update ? "yes" : "no (skip existing)"}`)
  console.log(`- all: ${all ? "yes" : "no"}`)
  console.log(`- openVariants: ${openVariants?.length ?? 0}`)
  console.log(`- variants: ${variants?.length ?? 0}`)
  console.log(`- openKeys: ${openKeys ? `${openKeys.modifiers.join("+")}+${openKeys.key}` : ""}`)
  console.log(`- steps: ${steps?.length ?? 0}`)
  console.log(`- openSteps: ${openSteps?.length ?? 0}`)
  console.log(`- freezeDate: ${freezeDate ?? ""}`)

  const finalNames = await resolveNames()
  console.log(`- names: ${finalNames.length}`)

  const invalidNames = finalNames.filter((name) => name.includes("."))
  if (invalidNames.length > 0) {
    throw new Error(
      `invalid route names (must not contain '.', got: ${invalidNames.join(", ")}). ` +
        `If you meant to update an output key like "chart-line-interactive.hover-mid", ` +
        `pass the base route name and use --variants=hover-mid. ` +
        `Example: node goldens/shadcn-web/scripts/extract-golden.mts --startServer ` +
        `--baseUrl=http://localhost:4020 --variants=hover-mid chart-line-interactive --update`
    )
  }

  if (startServer) {
    startedServer = await startNextServer(nextDir, baseUrl)
    await waitForHttpOk(
      `${baseUrl}/view/${style}/${finalNames[0] ?? "button-default"}`,
      Math.min(30_000, timeoutMs)
    )
  }

  const failures = await run({
    baseUrl,
    style,
    themes: themes.length > 0 ? themes : ["light", "dark"],
    modes: modes.length > 0 ? modes : ["closed"],
    names: finalNames,
    types,
    outDir,
    update,
    timeoutMs,
    viewportW,
    viewportH,
    deviceScaleFactor,
    freezeDate,
    openSelector,
    openAction,
    openKeys,
    steps,
    openSteps,
    openVariants,
    variants,
    mergeThemes: flags.mergeThemes === true || process.env.MERGE_THEMES === "1",
  })

  if (failures.length > 0) {
    console.error(`! golden extraction finished with ${failures.length} failures:`)
    for (const f of failures) {
      console.error(`  - ${f}`)
    }
    process.exitCode = 1
  }
} catch (error) {
  console.error(error)
  process.exit(1)
} finally {
  if (startedServer) {
    try {
      await startedServer.close()
    } catch (err) {
      console.error(`! failed to stop Next.js server: ${String(err)}`)
    }
  }
}
