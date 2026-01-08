import fs from "fs"
import os from "os"
import path from "path"
import puppeteer from "puppeteer"
import { fileURLToPath, pathToFileURL } from "url"

type Theme = "light" | "dark"

type GoldenOptions = {
  baseUrl: string
  style: string
  themes: Theme[]
  names: string[]
  types: string[]
  outDir: string
  update: boolean
  timeoutMs: number
}

type GoldenFile = {
  version: number
  style: string
  name: string
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

function resolveBrowserExecutablePath(): string | undefined {
  const envPath = process.env.PUPPETEER_EXECUTABLE_PATH
  if (envPath && fs.existsSync(envPath)) {
    return envPath
  }

  // Puppeteer browser cache (common in CI/dev): ~/.cache/puppeteer/chrome/**/chrome.exe
  // We don't rely on a specific revision; any working Chrome is fine for layout/style extraction.
  try {
    const cacheRoot = path.join(os.homedir(), ".cache", "puppeteer", "chrome")
    if (fs.existsSync(cacheRoot)) {
      const dirs = fs
        .readdirSync(cacheRoot, { withFileTypes: true })
        .filter((d) => d.isDirectory())
        .map((d) => d.name)
        .sort()
        .reverse()

      for (const dir of dirs) {
        const candidate = path.join(cacheRoot, dir, "chrome-win64", "chrome.exe")
        if (fs.existsSync(candidate)) {
          return candidate
        }
      }
    }
  } catch {
    // ignore
  }

  const candidates: string[] = []
  const programFiles = process.env.ProgramFiles
  const programFilesX86 = process.env["ProgramFiles(x86)"]
  const localAppData = process.env.LOCALAPPDATA

  if (programFiles) {
    candidates.push(
      path.join(programFiles, "Google", "Chrome", "Application", "chrome.exe")
    )
    candidates.push(
      path.join(programFiles, "Microsoft", "Edge", "Application", "msedge.exe")
    )
  }
  if (programFilesX86) {
    candidates.push(
      path.join(
        programFilesX86,
        "Google",
        "Chrome",
        "Application",
        "chrome.exe"
      )
    )
    candidates.push(
      path.join(
        programFilesX86,
        "Microsoft",
        "Edge",
        "Application",
        "msedge.exe"
      )
    )
  }
  if (localAppData) {
    candidates.push(
      path.join(localAppData, "Google", "Chrome", "Application", "chrome.exe")
    )
    candidates.push(
      path.join(localAppData, "Microsoft", "Edge", "Application", "msedge.exe")
    )
  }

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate
    }
  }

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

    const attrKeys = [
      "role",
      "aria-label",
      "aria-labelledby",
      "aria-describedby",
      "aria-checked",
      "aria-selected",
      "aria-expanded",
      "aria-pressed",
      "aria-controls",
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
      // Keep `id` and `class` as dedicated fields for easier selectors, but we also preserve them in attrs.
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

    function traverse(el, pathStr) {
      const rect = el.getBoundingClientRect();
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
          x: rect.x - rootRect.x,
          y: rect.y - rootRect.y,
          w: rect.width,
          h: rect.height,
        },
        computedStyle: collectStyle(el),
        text: collectText(el) || undefined,
        children: [],
      };

      const children = Array.from(el.children);
      for (let i = 0; i < children.length; i++) {
        const child = children[i];
        out.children.push(traverse(child, pathStr ? pathStr + "." + i : "" + i));
      }
      return out;
    }

    return {
      url: location.href,
      devicePixelRatio: window.devicePixelRatio,
      viewport: { w: window.innerWidth, h: window.innerHeight },
      root: traverse(root, ""),
    };
  })()`

  return await page.evaluate(expr)
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

  await page.waitForFunction(expr, { timeout: timeoutMs })
}

async function setThemeBeforeLoad(page: puppeteer.Page, theme: Theme) {
  await page.evaluateOnNewDocument((theme) => {
    localStorage.setItem("theme", theme)
  }, theme)
}

function repoRootFromScript(): string {
  const scriptPath = fileURLToPath(import.meta.url)
  const scriptDir = path.dirname(scriptPath)
  return path.resolve(scriptDir, "../../..")
}

const repoRoot = repoRootFromScript()

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

  const entryCss =
    entry.entryCSSFiles?.["[project]/apps/v4/app/layout"] ??
    entry.entryCSSFiles?.[`[project]/apps/v4/app/(view)/view/[style]/[name]/page`]

  if (Array.isArray(entryCss)) {
    for (const item of entryCss) {
      if (typeof item?.path === "string") {
        add(item.path)
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
      console.error("    1) pnpm -C repo-ref/ui/apps/v4 build")
      console.error("    2) pnpm -C repo-ref/ui/apps/v4 exec next start -p 4020")
      console.error(
        `  Then rerun with:\n    --baseUrl=http://localhost:4020`
      )
      throw new Error("refusing to run against dev server (use next start)")
    }
  }

  const executablePath = resolveBrowserExecutablePath()

  let browser: puppeteer.Browser
  try {
    browser = await puppeteer.launch({
      ...(executablePath ? { executablePath } : {}),
      defaultViewport: {
        width: 1440,
        height: 900,
        deviceScaleFactor: 2,
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
      await page.emulateMediaFeatures([
        { name: "prefers-reduced-motion", value: "reduce" },
      ])
      await setThemeBeforeLoad(page, options.themes[0] ?? "light")
      const url = `${options.baseUrl}/view/${options.style}/button-default`
      await page.goto(url, { waitUntil: "networkidle2" })
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
      await page.emulateMediaFeatures([
        { name: "prefers-reduced-motion", value: "reduce" },
      ])
      await setThemeBeforeLoad(page, theme)
      pagesByTheme[theme] = page
    }

    for (const name of options.names) {
      const outPath = path.join(options.outDir, `${name}.json`)
      if (!options.update && fs.existsSync(outPath)) {
        continue
      }

      const out: GoldenFile = {
        version: 1,
        style: options.style,
        name,
        themes: {},
      }

      let ok = true
      for (const theme of options.themes) {
        const url = `${options.baseUrl}/view/${options.style}/${name}`
        try {
          const page = pagesByTheme[theme]

          await page.goto(url, { waitUntil: "networkidle2" })
          await page.waitForSelector("[data-fret-golden-target]", { timeout: 30000 })
          await injectCssLinks(page, cssInjectionUrls)

          await page.evaluate(() => {
            const indicator = document.querySelector("[data-tailwind-indicator]")
            if (indicator) indicator.remove()
          })

          await waitForFonts(page, Math.min(2000, options.timeoutMs))
          await waitForShadcnStyles(page, Math.min(30000, options.timeoutMs))

          const extracted = await extractOne(page)
          // Normalize a few floats that may slip through as high precision.
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          ;(extracted as any).devicePixelRatio = round3(
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            (extracted as any).devicePixelRatio
          )

          ;(out.themes as Record<string, unknown>)[theme] = extracted
        } catch (error) {
          ok = false
          const msg = `${name} (${theme}): ${String(error)}`
          failures.push(msg)
          console.error(`! failed ${name} (${theme})`)
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
        console.log(`- wrote ${path.relative(process.cwd(), outPath)}`)
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

const baseUrl =
  (typeof flags.baseUrl === "string" ? flags.baseUrl : undefined) ??
  process.env.BASE_URL ??
  "http://localhost:4000"

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

const update = flags.update === true || process.env.UPDATE_GOLDENS === "1"

const defaultNames = ["button-default", "tabs-demo"]
const all = flags.all === true || process.env.ALL_GOLDENS === "1"

async function resolveNames(): Promise<string[]> {
  if (!all) {
    return names.length > 0 ? names : defaultNames
  }

  const mod = await import(
    pathToFileURL(
      path.join(repoRoot, "repo-ref", "ui", "apps", "v4", "registry", "__index__.tsx")
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

try {
  console.log(`?? shadcn web golden extract`)
  console.log(`- baseUrl: ${baseUrl}`)
  console.log(`- style: ${style}`)
  console.log(`- themes: ${themes.join(", ")}`)
  console.log(`- types: ${types.join(", ")}`)
  console.log(`- outDir: ${outDir}`)
  console.log(`- timeoutMs: ${timeoutMs}`)
  console.log(`- update: ${update ? "yes" : "no (skip existing)"}`)
  console.log(`- all: ${all ? "yes" : "no"}`)

  const finalNames = await resolveNames()
  console.log(`- names: ${finalNames.length}`)

  const failures = await run({
    baseUrl,
    style,
    themes: themes.length > 0 ? themes : ["light", "dark"],
    names: finalNames,
    types,
    outDir,
    update,
    timeoutMs,
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
}
