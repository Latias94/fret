import fs from "fs"
import os from "os"
import path from "path"
import type puppeteer from "puppeteer"
import { createRequire } from "module"
import { fileURLToPath, pathToFileURL } from "url"

type Theme = "light" | "dark"

type Action =
  | { kind: "load"; url: string }
  | { kind: "click"; target: string }
  | { kind: "press"; key: string }
  | { kind: "hover"; target: string }

type Step = {
  action: Action
  snapshot: {
    focus: DomFocus | null
    dom: DomNode
    ax: unknown | null
  }
}

type TimelineGolden = {
  version: 1
  baseUrl: string
  base: "radix"
  style: string
  baseColor: string
  theme: Theme
  radius: string
  font: string
  iconLibrary: string
  menuAccent: string
  menuColor: string
  item: string
  primitive: string
  scenario: string
  url: string
  steps: Step[]
}

type DomFocus = {
  tag: string
  attrs: Record<string, string>
  text?: string
  path: number[]
}

type DomNode = {
  tag: string
  path: number[]
  attrs: Record<string, string>
  text?: string
  children: DomNode[]
}

type Scenario = {
  primitive: string
  scenario: string
  item: string
  run: (ctx: ScenarioContext) => Promise<void>
}

type ScenarioContext = {
  page: puppeteer.Page
  baseUrl: string
  url: string
  steps: Step[]
  timeoutMs: number
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

function ensureDir(dir: string) {
  fs.mkdirSync(dir, { recursive: true })
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

async function sleep(ms: number) {
  await new Promise((r) => setTimeout(r, ms))
}

function resolveBrowserExecutablePath(): string | undefined {
  const envPath = process.env.PUPPETEER_EXECUTABLE_PATH
  if (envPath && fs.existsSync(envPath)) {
    return envPath
  }

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

async function snapshotDom(page: puppeteer.Page): Promise<{
  focus: DomFocus | null
  dom: DomNode
}> {
  const expr = `(() => {
    const interestingAttrPrefixes = ["aria-"];
    const interestingAttrs = new Set([
      "role",
      "tabindex",
      "data-state",
      "data-disabled",
      "data-highlighted",
      "data-orientation",
      "data-side",
      "data-align",
      "data-radix-collection-item",
    ]);

    const attrWhitelist = (el) => {
      const out = {};
      for (const name of el.getAttributeNames()) {
        if (interestingAttrs.has(name) || interestingAttrPrefixes.some((p) => name.startsWith(p))) {
          const v = el.getAttribute(name);
          if (v != null) out[name] = v;
        }
      }
      return out;
    };

    const textOf = (el) => {
      const tag = el.tagName;
      if (tag === "HTML" || tag === "BODY" || tag === "SCRIPT" || tag === "STYLE") return null;
      const t = (el.textContent || "").trim();
      if (!t) return null;
      if (t.length > 120) return t.slice(0, 120) + "…";
      return t;
    };

    const pathFromBody = (el) => {
      const path = [];
      let cur = el;
      while (cur && cur !== document.body) {
        const parent = cur.parentElement;
        if (!parent) break;
        const idx = Array.prototype.indexOf.call(parent.children, cur);
        path.unshift(idx);
        cur = parent;
      }
      return path;
    };

    const isInteresting = (el) => {
      if (!el) return false;
      const tag = el.tagName;
      if (tag === "BUTTON" || tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
      if (el.hasAttribute("role")) return true;
      for (const name of el.getAttributeNames()) {
        if (interestingAttrs.has(name) || interestingAttrPrefixes.some((p) => name.startsWith(p))) return true;
      }
      return false;
    };

    const active = document.activeElement instanceof HTMLElement ? document.activeElement : null;

    const build = (el) => {
      const children = Array.from(el.children);
      const builtChildren = [];
      let anyChildIncluded = false;
      for (let i = 0; i < children.length; i++) {
        const child = children[i];
        const built = build(child);
        if (built) {
          builtChildren.push(built);
          anyChildIncluded = true;
        }
      }

      const includeSelf = isInteresting(el) || (active && el === active) || anyChildIncluded;
      if (!includeSelf) return null;

      const node = {
        tag: el.tagName.toLowerCase(),
        path: pathFromBody(el),
        attrs: attrWhitelist(el),
        children: builtChildren,
      };

      const txt = textOf(el);
      if (txt) node.text = txt;
      return node;
    };

    const root = document.body;
    const dom = build(root) || { tag: "body", path: [], attrs: {}, children: [] };

    let focus = null;
    if (active) {
      focus = {
        tag: active.tagName.toLowerCase(),
        path: pathFromBody(active),
        attrs: attrWhitelist(active),
      };
      const t = textOf(active);
      if (t) focus.text = t;
    }

    return { focus, dom };
  })()`

  return await page.evaluate(expr)
}

async function snapshotStep(page: puppeteer.Page): Promise<Step["snapshot"]> {
  const dom = await snapshotDom(page)
  let ax: unknown | null = null
  try {
    ax = await page.accessibility.snapshot({ interestingOnly: true })
  } catch {
    ax = null
  }
  return { focus: dom.focus, dom: dom.dom, ax }
}

async function pushStep(ctx: ScenarioContext, action: Action) {
  const snapshot = await snapshotStep(ctx.page)
  ctx.steps.push({ action, snapshot })
}

async function clickFirstByText(
  page: puppeteer.Page,
  selector: string,
  containsText: string
) {
  const handles = await page.$$(selector)
  for (const handle of handles) {
    const text = await handle.evaluate((el) =>
      (el.textContent || "").trim()
    )
    if (text.includes(containsText)) {
      await handle.click()
      return
    }
  }
  throw new Error(`no element for ${selector} containing text: ${containsText}`)
}

async function clickUntilRoleAppears(
  page: puppeteer.Page,
  selector: string,
  role: string,
  timeoutMs: number
) {
  const handles = await page.$$(selector)
  const perTryTimeout = Math.min(1500, timeoutMs)
  for (const handle of handles) {
    await handle.click()
    try {
      await waitForRole(page, role, true, perTryTimeout)
      return
    } catch {
      try {
        await page.keyboard.press("Escape")
        await sleep(50)
      } catch {
        // ignore
      }
    }
  }
  throw new Error(`no click target for ${selector} that opens role=${role}`)
}

async function waitForRole(page: puppeteer.Page, role: string, present: boolean, timeoutMs: number) {
  const expr = `(() => {
    const found = !!document.querySelector('[role=\"${role}\"]');
    return ${present ? "found" : "!found"};
  })()`
  await page.waitForFunction(expr, { timeout: timeoutMs })
}

function previewUrl(baseUrl: string, item: string, config: Record<string, string>) {
  const params = new URLSearchParams(config)
  return `${baseUrl}/preview/radix/${item}?${params.toString()}`
}

const scenarios: Scenario[] = [
  {
    primitive: "dialog",
    scenario: "open-close",
    item: "dialog-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickUntilRoleAppears(ctx.page, "button", "dialog", ctx.timeoutMs)
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "dialog-trigger" })

      await ctx.page.keyboard.press("Escape")
      await waitForRole(ctx.page, "dialog", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "Escape" })
    },
  },
  {
    primitive: "dropdown-menu",
    scenario: "open-navigate-select",
    item: "dropdown-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      // Prefer aria-haspopup=menu when present; otherwise fall back to first "Open" button.
      const trigger =
        (await ctx.page.$('[aria-haspopup=\"menu\"]')) ||
        (await ctx.page.$('button'))
      if (!trigger) throw new Error("missing dropdown menu trigger")
      await trigger.click()
      await sleep(50)
      await waitForRole(ctx.page, "menu", true, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "dropdown-trigger" })

      await ctx.page.keyboard.press("ArrowDown")
      await ctx.page.keyboard.press("Enter")
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "ArrowDown,Enter" })
    },
  },
  {
    primitive: "select",
    scenario: "open-navigate-select",
    item: "select-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      const trigger =
        (await ctx.page.$('[role=\"combobox\"]')) ||
        (await ctx.page.$('[aria-haspopup=\"listbox\"]')) ||
        (await ctx.page.$('button'))
      if (!trigger) throw new Error("missing select trigger")
      await trigger.click()
      await sleep(50)
      await waitForRole(ctx.page, "listbox", true, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "select-trigger" })

      await ctx.page.keyboard.press("ArrowDown")
      await ctx.page.keyboard.press("ArrowDown")
      await ctx.page.keyboard.press("Enter")
      await sleep(50)
      await waitForRole(ctx.page, "listbox", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "ArrowDown,ArrowDown,Enter" })
    },
  },
  {
    primitive: "tooltip",
    scenario: "hover-show-hide",
    item: "tooltip-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      const trigger =
        (await ctx.page.$('[aria-describedby]')) ||
        (await ctx.page.$('[data-state]')) ||
        (await ctx.page.$('button')) ||
        (await ctx.page.$('a'))
      if (!trigger) throw new Error("missing tooltip trigger")
      await trigger.hover()
      await sleep(150)
      await waitForRole(ctx.page, "tooltip", true, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "hover", target: "tooltip-trigger" })

      await ctx.page.mouse.move(0, 0)
      await sleep(150)
      await waitForRole(ctx.page, "tooltip", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "hover", target: "mouse-away" })
    },
  },
]

function repoRootFromScript(): string {
  const scriptPath = fileURLToPath(import.meta.url)
  const scriptDir = path.dirname(scriptPath)
  return path.resolve(scriptDir, "../../..")
}

const repoRoot = repoRootFromScript()

async function loadPuppeteer(): Promise<typeof import("puppeteer")> {
  const require = createRequire(import.meta.url)
  const entry = require.resolve("puppeteer", {
    paths: [path.join(repoRoot, "repo-ref", "ui")],
  })
  const mod = await import(pathToFileURL(entry).href)
  return ((mod as any).default ?? mod) as typeof import("puppeteer")
}

const { flags, names } = parseArgs(process.argv.slice(2))

const baseUrl =
  (typeof flags.baseUrl === "string" ? flags.baseUrl : undefined) ??
  process.env.BASE_URL ??
  "http://localhost:4020"

const outDir =
  (typeof flags.outDir === "string" ? flags.outDir : undefined) ??
  process.env.OUT_DIR ??
  path.join(repoRoot, "goldens", "radix-web", "v4", "radix-vega")

const update = flags.update === true || process.env.UPDATE_GOLDENS === "1"
const all = flags.all === true || process.env.ALL_GOLDENS === "1"

const timeoutMs =
  Number(
    (typeof flags.timeoutMs === "string" ? flags.timeoutMs : undefined) ??
      process.env.TIMEOUT_MS ??
      "60000"
  ) || 60000

const theme =
  (typeof flags.theme === "string" ? flags.theme : undefined) ??
  (process.env.THEME as Theme | undefined) ??
  "light"

const config = {
  base: "radix",
  style:
    (typeof flags.style === "string" ? flags.style : undefined) ??
    process.env.STYLE ??
    "vega",
  baseColor:
    (typeof flags.baseColor === "string" ? flags.baseColor : undefined) ??
    process.env.BASE_COLOR ??
    "neutral",
  theme:
    (typeof flags.dsTheme === "string" ? flags.dsTheme : undefined) ??
    process.env.DS_THEME ??
    "neutral",
  radius:
    (typeof flags.radius === "string" ? flags.radius : undefined) ??
    process.env.RADIUS ??
    "default",
  font:
    (typeof flags.font === "string" ? flags.font : undefined) ??
    process.env.FONT ??
    "inter",
  iconLibrary:
    (typeof flags.iconLibrary === "string" ? flags.iconLibrary : undefined) ??
    process.env.ICON_LIBRARY ??
    "lucide",
  menuAccent:
    (typeof flags.menuAccent === "string" ? flags.menuAccent : undefined) ??
    process.env.MENU_ACCENT ??
    "subtle",
  menuColor:
    (typeof flags.menuColor === "string" ? flags.menuColor : undefined) ??
    process.env.MENU_COLOR ??
    "default",
}

function selectedScenarios(): Scenario[] {
  if (!all) {
    const wanted = new Set(names)
    if (wanted.size === 0) {
      return scenarios.filter((s) =>
        ["dialog-example", "select-example", "dropdown-menu-example"].includes(s.item)
      )
    }
    return scenarios.filter((s) => wanted.has(s.item) || wanted.has(s.primitive))
  }
  return scenarios
}

async function main() {
  const puppeteer = await loadPuppeteer()
  ensureDir(outDir)
  const executablePath = resolveBrowserExecutablePath()

  const selected = selectedScenarios()
  console.log("?? radix web behavior goldens")
  console.log(`- baseUrl: ${baseUrl}`)
  console.log(`- outDir: ${outDir}`)
  console.log(`- theme: ${theme}`)
  console.log(`- update: ${update ? "yes" : "no (skip existing)"}`)
  console.log(`- scenarios: ${selected.length}`)

  let browser: puppeteer.Browser
  browser = await puppeteer.launch({
    ...(executablePath ? { executablePath } : {}),
    defaultViewport: { width: 1280, height: 800, deviceScaleFactor: 2 },
  })

  try {
    const failures: string[] = []
    for (const s of selected) {
      const url = previewUrl(baseUrl, s.item, config)
      const outPath = path.join(
        outDir,
        `${s.item}.${s.primitive}.${s.scenario}.${theme}.json`
      )
      if (!update && fs.existsSync(outPath)) {
        continue
      }

      const page = await browser.newPage()
      page.setDefaultTimeout(timeoutMs)

      // Ensure theme is stable.
      await page.evaluateOnNewDocument((theme) => {
        localStorage.setItem("theme", theme)
      }, theme)

      const ctx: ScenarioContext = {
        page,
        baseUrl,
        url,
        steps: [],
        timeoutMs,
      }

      try {
        await page.goto(url, { waitUntil: "networkidle2" })
        // Wait for any buttons/inputs to exist as a rough “hydration finished” signal.
        await page.waitForSelector("body", { timeout: 30000 })
        await sleep(50)
        await s.run(ctx)

        const golden: TimelineGolden = {
          version: 1,
          baseUrl,
          base: "radix",
          style: config.style,
          baseColor: config.baseColor,
          theme,
          radius: config.radius,
          font: config.font,
          iconLibrary: config.iconLibrary,
          menuAccent: config.menuAccent,
          menuColor: config.menuColor,
          item: s.item,
          primitive: s.primitive,
          scenario: s.scenario,
          url,
          steps: ctx.steps,
        }

        writeIfChanged(outPath, golden, update)
        console.log(`- wrote ${path.relative(process.cwd(), outPath)}`)
      } catch (error) {
        const msg = `${s.item}.${s.primitive}.${s.scenario}: ${String(error)}`
        failures.push(msg)
        console.error(`! failed ${msg}`)
      } finally {
        await page.close()
      }
    }

    if (failures.length > 0) {
      console.error(`! finished with ${failures.length} failures:`)
      for (const f of failures) console.error(`  - ${f}`)
      process.exitCode = 1
    }
  } finally {
    await browser.close()
  }
}

await main()
