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
  rect?: { x: number; y: number; w: number; h: number }
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

const SETTLE_MS = 50

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
      "data-slot",
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

      try {
        const r = el.getBoundingClientRect();
        // We intentionally keep viewport-relative rects so they can be compared to Fret window
        // coordinates without additional normalization.
        node.rect = {
          x: r.x,
          y: r.y,
          w: r.width,
          h: r.height,
        };
      } catch {
        // ignore
      }

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

async function findFirstByText(
  page: puppeteer.Page,
  selector: string,
  containsText: string
) {
  const handles = await page.$$(selector)
  for (const handle of handles) {
    const text = await handle.evaluate((el) => (el.textContent || "").trim())
    if (text.includes(containsText)) return handle
  }
  throw new Error(`no element for ${selector} containing text: ${containsText}`)
}

async function centerOf(handle: puppeteer.ElementHandle<Element>) {
  return await handle.evaluate((el) => {
    const r = el.getBoundingClientRect()
    return { x: r.left + r.width / 2, y: r.top + r.height / 2 }
  })
}

async function moveMouseTo(handle: puppeteer.ElementHandle<Element>, page: puppeteer.Page) {
  const { x, y } = await centerOf(handle)
  await page.mouse.move(x, y)
}

async function clickFirst(page: puppeteer.Page, selector: string) {
  const handle = await page.$(selector)
  if (!handle) throw new Error(`missing element: ${selector}`)
  await handle.click()
}

async function hoverFirstByText(
  page: puppeteer.Page,
  selector: string,
  containsText: string
) {
  const handles = await page.$$(selector)
  for (const handle of handles) {
    const text = await handle.evaluate((el) => (el.textContent || "").trim())
    if (text.includes(containsText)) {
      await handle.hover()
      return
    }
  }
  throw new Error(`no element for ${selector} containing text: ${containsText}`)
}

async function hoverExampleWithinSelectorByText(
  page: puppeteer.Page,
  title: string,
  itemSelector: string,
  containsText: string
) {
  const example = await findExampleByTitle(page, title)
  const items = await example.$$(itemSelector)
  for (const item of items) {
    const text = await item.evaluate((el) => (el.textContent || "").trim())
    if (!text.includes(containsText)) continue
    await item.hover()
    return
  }
  throw new Error(
    `no element for selector=${itemSelector} in example title=${title} containing text: ${containsText}`
  )
}

async function findExampleByTitle(page: puppeteer.Page, title: string) {
  const examples = await page.$$('[data-slot="example"]')
  for (const example of examples) {
    const titleText = await example.evaluate((el) =>
      (el.firstElementChild?.textContent || "").trim()
    )
    if (titleText === title) return example
  }
  throw new Error(`missing example title=${title}`)
}

async function clickExampleTrigger(
  page: puppeteer.Page,
  title: string,
  triggerSelector: string
) {
  const example = await findExampleByTitle(page, title)
  const trigger = await example.$(triggerSelector)
  if (!trigger) {
    throw new Error(
      `missing trigger selector=${triggerSelector} for example title=${title}`
    )
  }
  await trigger.click()
}

async function focusExampleTrigger(
  page: puppeteer.Page,
  title: string,
  triggerSelector: string
) {
  const example = await findExampleByTitle(page, title)
  const trigger = await example.$(triggerSelector)
  if (!trigger) {
    throw new Error(
      `missing trigger selector=${triggerSelector} for example title=${title}`
    )
  }
  await trigger.focus()
}

async function rightClickExampleTrigger(
  page: puppeteer.Page,
  title: string,
  triggerSelector: string
) {
  const example = await findExampleByTitle(page, title)
  const trigger = await example.$(triggerSelector)
  if (!trigger) {
    throw new Error(
      `missing trigger selector=${triggerSelector} for example title=${title}`
    )
  }
  await trigger.click({ button: "right" })
}

async function clickExampleWithinSelectorByText(
  page: puppeteer.Page,
  title: string,
  itemSelector: string,
  containsText: string
) {
  const example = await findExampleByTitle(page, title)
  const items = await example.$$(itemSelector)
  for (const item of items) {
    const text = await item.evaluate((el) => (el.textContent || "").trim())
    if (!text.includes(containsText)) continue
    await item.click()
    return
  }
  throw new Error(
    `no element for selector=${itemSelector} in example title=${title} containing text: ${containsText}`
  )
}

async function clickWithinSelectorByText(
  page: puppeteer.Page,
  containerSelector: string,
  itemSelector: string,
  containsText: string
) {
  const container = await page.$(containerSelector)
  if (!container) throw new Error(`missing container: ${containerSelector}`)
  const items = await container.$$(itemSelector)
  for (const item of items) {
    const text = await item.evaluate((el) => (el.textContent || "").trim())
    if (!text.includes(containsText)) continue
    await item.click()
    return
  }
  throw new Error(
    `no element in ${containerSelector} for ${itemSelector} containing text: ${containsText}`
  )
}

async function press(page: puppeteer.Page, key: string) {
  await page.keyboard.press(key)
}

async function pressChord(page: puppeteer.Page, keys: string[]) {
  for (const key of keys) {
    await press(page, key)
  }
}

type ActiveElementInfo = {
  tag: string
  role: string | null
  inMenu: boolean
  text: string
}

async function activeElementInfo(page: puppeteer.Page): Promise<ActiveElementInfo> {
  return await page.evaluate(() => {
    const el = document.activeElement
    const htmlEl = el instanceof HTMLElement ? el : null
    const role = htmlEl?.getAttribute("role") ?? null
    const inMenu = htmlEl ? Boolean(htmlEl.closest('[role="menu"]')) : false
    const text = htmlEl ? (htmlEl.textContent || "").trim() : ""
    return {
      tag: htmlEl ? htmlEl.tagName.toLowerCase() : "unknown",
      role,
      inMenu,
      text: text.length > 120 ? text.slice(0, 120) + "…" : text,
    }
  })
}

async function pressUntilActiveElementContainsText(
  page: puppeteer.Page,
  key: string,
  containsText: string,
  maxPresses: number
): Promise<string[]> {
  const pressed: string[] = []
  for (let i = 0; i < maxPresses; i++) {
    const active = await activeElementInfo(page)
    if (
      active.inMenu &&
      active.role &&
      active.role.startsWith("menuitem") &&
      active.text.includes(containsText)
    ) {
      return pressed
    }
    await press(page, key)
    pressed.push(key)
    await sleep(SETTLE_MS)
  }
  const active = await activeElementInfo(page)
  throw new Error(
    `active element did not match role=menuitem* and contain text=${containsText} after ${maxPresses} presses of ${key}; last active=${JSON.stringify(
      active
    )}`
  )
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
    try {
      await handle.click()
    } catch {
      continue
    }
    try {
      await waitForRole(page, role, true, perTryTimeout)
      return
    } catch {
      try {
        await page.keyboard.press("Escape")
        await sleep(SETTLE_MS)
      } catch {
        // ignore
      }
    }
  }
  throw new Error(`no click target for ${selector} that opens role=${role}`)
}

async function hoverUntilRoleAppears(
  page: puppeteer.Page,
  selector: string,
  role: string,
  timeoutMs: number
) {
  const handles = await page.$$(selector)
  const perTryTimeout = Math.min(1500, timeoutMs)
  for (const handle of handles) {
    try {
      await handle.hover()
    } catch {
      continue
    }
    try {
      await waitForRole(page, role, true, perTryTimeout)
      return
    } catch {
      try {
        await page.mouse.move(0, 0)
        await sleep(50)
      } catch {
        // ignore
      }
    }
  }
  throw new Error(`no hover target for ${selector} that opens role=${role}`)
}

async function rightClickUntilRoleAppears(
  page: puppeteer.Page,
  selector: string,
  role: string,
  timeoutMs: number
) {
  const handles = await page.$$(selector)
  const perTryTimeout = Math.min(1500, timeoutMs)
  for (const handle of handles) {
    let box: puppeteer.BoundingBox | null = null
    try {
      box = await handle.boundingBox()
    } catch {
      box = null
    }
    try {
      if (box) {
        await page.mouse.click(box.x + 5, box.y + 5, { button: "right" as any })
      } else {
        await page.click("body", { button: "right" as any })
      }
    } catch {
      continue
    }
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
  throw new Error(`no right-click target for ${selector} that opens role=${role}`)
}

async function waitForRole(page: puppeteer.Page, role: string, present: boolean, timeoutMs: number) {
  const expr = `(() => {
    const found = !!document.querySelector('[role=\"${role}\"]');
    return ${present ? "found" : "!found"};
  })()`
  await page.waitForFunction(expr, { timeout: timeoutMs })
}

async function waitForSelectorPresent(
  page: puppeteer.Page,
  selector: string,
  present: boolean,
  timeoutMs: number
) {
  const sel = JSON.stringify(selector)
  const expr = `(() => {
    const found = !!document.querySelector(${sel});
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
    primitive: "accordion",
    scenario: "toggle-first",
    item: "accordion-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      await clickFirst(ctx.page, '[data-slot="accordion-trigger"]')
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "accordion-trigger" })
      await clickFirst(ctx.page, '[data-slot="accordion-trigger"]')
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "accordion-trigger" })
    },
  },
  {
    primitive: "dialog",
    scenario: "open-close",
    item: "dialog-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickUntilRoleAppears(ctx.page, "button", "dialog", ctx.timeoutMs)
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "dialog-trigger" })

      await press(ctx.page, "Escape")
      await waitForRole(ctx.page, "dialog", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "Escape" })
    },
  },
  {
    primitive: "popover",
    scenario: "open-close",
    item: "popover-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      await clickUntilRoleAppears(ctx.page, "button", "dialog", ctx.timeoutMs)
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "popover-trigger" })
      await press(ctx.page, "Escape")
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

      await pressChord(ctx.page, ["ArrowDown", "Enter"])
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "ArrowDown,Enter" })
    },
  },
  {
    primitive: "dropdown-menu",
    scenario: "submenu-hover-select",
    item: "dropdown-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="dropdown-menu-trigger"]'
      )
      await sleep(50)
      await waitForSelectorPresent(ctx.page, '[data-slot="dropdown-menu-content"]', true, ctx.timeoutMs).catch(
        (err) => {
          throw new Error(`submenu-hover-select: menu content did not appear: ${String(err)}`)
        }
      )
      await pushStep(ctx, { kind: "click", target: "dropdown-menu:with-submenu" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="dropdown-menu-sub-trigger"]',
        "Invite users"
      )
      await sleep(50)
      await waitForSelectorPresent(ctx.page, '[data-slot="dropdown-menu-sub-content"]', true, ctx.timeoutMs).catch(
        (err) => {
          throw new Error(`submenu-hover-select: submenu content did not appear: ${String(err)}`)
        }
      )
      await pushStep(ctx, { kind: "hover", target: "dropdown-menu-sub-trigger:Invite users" })

      const content = await ctx.page.$('[data-slot="dropdown-menu-sub-content"]')
      if (!content) throw new Error("missing dropdown-menu-sub-content after hover")
      await content.hover()
      await sleep(50)
      await waitForSelectorPresent(ctx.page, '[data-slot="dropdown-menu-sub-content"]', true, ctx.timeoutMs)
      await pushStep(ctx, { kind: "hover", target: "dropdown-menu-sub-content" })

      await clickWithinSelectorByText(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        '[role="menuitem"]',
        "Email"
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs)).catch(
        (err) => {
          throw new Error(`submenu-hover-select: menu did not close after select: ${String(err)}`)
        }
      )
      await pushStep(ctx, { kind: "click", target: "dropdown-menu-sub-item:Email" })
    },
  },
  {
    primitive: "dropdown-menu",
    scenario: "submenu-grace-corridor",
    item: "dropdown-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="dropdown-menu-trigger"]'
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: menu content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "click", target: "dropdown-menu:with-submenu" })

      const subTrigger = await findFirstByText(
        ctx.page,
        '[data-slot="dropdown-menu-sub-trigger"]',
        "Invite users"
      )
      await moveMouseTo(subTrigger, ctx.page)
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: submenu content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, {
        kind: "hover",
        target: "dropdown-menu-sub-trigger:Invite users",
      })

      const subContent = await ctx.page.$('[data-slot="dropdown-menu-sub-content"]')
      if (!subContent) throw new Error("missing dropdown-menu-sub-content after hover")

      const { x: fromX, y: fromY } = await centerOf(subTrigger)
      const { x: toX, y: toY } = await centerOf(subContent)
      const mid = { x: (fromX + toX) / 2, y: (fromY + toY) / 2 }

      await ctx.page.mouse.move(mid.x, mid.y)
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "submenu-grace:mid" })

      await ctx.page.mouse.move(toX, toY)
      await sleep(200)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: submenu content disappeared: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "hover", target: "submenu-grace:inside" })

      await clickWithinSelectorByText(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        '[role="menuitem"]',
        "Email"
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "dropdown-menu-sub-item:Email" })
    },
  },
  {
    primitive: "dropdown-menu",
    scenario: "submenu-unsafe-leave",
    item: "dropdown-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="dropdown-menu-trigger"]'
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "click", target: "dropdown-menu:with-submenu" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="dropdown-menu-sub-trigger"]',
        "Invite users"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, {
        kind: "hover",
        target: "dropdown-menu-sub-trigger:Invite users",
      })

      await hoverFirstByText(ctx.page, '[data-slot="dropdown-menu-item"]', "Team")
      await sleep(200)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        false,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-unsafe-leave: submenu content did not close after leaving: ${String(
            err
          )}`
        )
      })
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "dropdown-menu-item:Team" })
    },
  },
  {
    primitive: "dropdown-menu",
    scenario: "submenu-keyboard-open-close",
    item: "dropdown-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await focusExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="dropdown-menu-trigger"]'
      )
      await press(ctx.page, "ArrowDown")
      await sleep(50)
      await waitForRole(ctx.page, "menu", true, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "ArrowDown" })

      await pressChord(ctx.page, ["ArrowDown", "ArrowRight"])
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: "ArrowDown,ArrowRight" })

      await press(ctx.page, "ArrowLeft")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        false,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: "ArrowLeft" })
    },
  },
  {
    primitive: "dropdown-menu",
    scenario: "submenu-arrowleft-escape-close",
    item: "dropdown-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await focusExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="dropdown-menu-trigger"]'
      )
      await press(ctx.page, "ArrowDown")
      await sleep(50)
      await waitForRole(ctx.page, "menu", true, Math.min(15000, ctx.timeoutMs)).catch(
        (err) => {
          throw new Error(
            `submenu-arrowleft-escape-close: menu did not open after ArrowDown: ${String(err)}`
          )
        }
      )
      await pushStep(ctx, { kind: "press", key: "ArrowDown" })

      const pressed = await pressUntilActiveElementContainsText(
        ctx.page,
        "ArrowDown",
        "Invite users",
        20
      )
      await press(ctx.page, "ArrowRight")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-arrowleft-escape-close: submenu did not open after ArrowRight: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "press", key: [...pressed, "ArrowRight"].join(",") })

      await press(ctx.page, "ArrowLeft")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        false,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-arrowleft-escape-close: submenu did not close after ArrowLeft: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "press", key: "ArrowLeft" })

      await press(ctx.page, "Escape")
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs)).catch(
        (err) => {
          throw new Error(
            `submenu-arrowleft-escape-close: menu did not close after Escape: ${String(err)}`
          )
        }
      )
      await pushStep(ctx, { kind: "press", key: "Escape" })
    },
  },
  {
    primitive: "dropdown-menu",
    scenario: "outside-click-close",
    item: "dropdown-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="dropdown-menu-trigger"]'
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "dropdown-menu:with-submenu" })

      // Click outside to dismiss (non-click-through).
      await ctx.page.mouse.click(5, 5)
      await sleep(100)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "outside" })
    },
  },
  {
    primitive: "dropdown-menu",
    scenario: "submenu-outside-click-close",
    item: "dropdown-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="dropdown-menu-trigger"]'
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "dropdown-menu:with-submenu" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="dropdown-menu-sub-trigger"]',
        "Invite users"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="dropdown-menu-sub-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "hover", target: "dropdown-menu-sub-trigger:Invite users" })

      // Click outside to dismiss both layers.
      await ctx.page.mouse.click(5, 5)
      await sleep(100)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "outside" })
    },
  },
  {
    primitive: "context-menu",
    scenario: "context-open-close",
    item: "context-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await rightClickUntilRoleAppears(
        ctx.page,
        "button, a, [role], [data-state], div",
        "menu",
        ctx.timeoutMs
      )
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "contextmenu:rightclick" })

      await press(ctx.page, "Escape")
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "Escape" })
    },
  },
  {
    primitive: "context-menu",
    scenario: "submenu-keyboard-open-close",
    item: "context-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await rightClickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="context-menu-trigger"]'
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", true, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "context-menu:with-submenu" })

      const pressed = await pressUntilActiveElementContainsText(
        ctx.page,
        "ArrowDown",
        "More Tools",
        20
      )
      await press(ctx.page, "ArrowRight")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: [...pressed, "ArrowRight"].join(",") })

      await press(ctx.page, "ArrowLeft")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        false,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: "ArrowLeft" })
    },
  },
  {
    primitive: "context-menu",
    scenario: "submenu-arrowleft-escape-close",
    item: "context-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await rightClickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="context-menu-trigger"]'
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", true, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "context-menu:with-submenu" })

      const pressed = await pressUntilActiveElementContainsText(
        ctx.page,
        "ArrowDown",
        "More Tools",
        20
      )
      await press(ctx.page, "ArrowRight")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: [...pressed, "ArrowRight"].join(",") })

      await press(ctx.page, "ArrowLeft")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        false,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: "ArrowLeft" })

      await press(ctx.page, "Escape")
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "Escape" })
    },
  },
  {
    primitive: "context-menu",
    scenario: "outside-click-close",
    item: "context-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await rightClickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="context-menu-trigger"]'
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", true, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "context-menu:with-submenu" })

      // Click outside to dismiss (non-click-through).
      await ctx.page.mouse.click(5, 5)
      await sleep(100)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "outside" })
    },
  },
  {
    primitive: "context-menu",
    scenario: "submenu-outside-click-close",
    item: "context-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await rightClickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="context-menu-trigger"]'
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", true, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "context-menu:with-submenu" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="context-menu-sub-trigger"]',
        "More Tools"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "hover", target: "context-menu-sub-trigger:More Tools" })

      // Click outside to dismiss both layers.
      await ctx.page.mouse.click(5, 5)
      await sleep(100)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "outside" })
    },
  },
  {
    primitive: "context-menu",
    scenario: "submenu-hover-select",
    item: "context-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await rightClickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="context-menu-trigger"]'
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", true, Math.min(15000, ctx.timeoutMs)).catch(
        (err) => {
          throw new Error(
            `submenu-hover-select: context menu content did not appear: ${String(err)}`
          )
        }
      )
      await pushStep(ctx, { kind: "click", target: "context-menu:with-submenu" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="context-menu-sub-trigger"]',
        "More Tools"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-hover-select: context submenu content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "hover", target: "context-menu-sub-trigger:More Tools" })

      await clickWithinSelectorByText(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        '[role="menuitem"]',
        "Save Page"
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs)).catch(
        (err) => {
          throw new Error(
            `submenu-hover-select: context menu did not close after select: ${String(err)}`
          )
        }
      )
      await pushStep(ctx, { kind: "click", target: "context-menu-sub-item:Save Page" })
    },
  },
  {
    primitive: "context-menu",
    scenario: "submenu-grace-corridor",
    item: "context-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await rightClickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="context-menu-trigger"]'
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: context menu content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "click", target: "context-menu:with-submenu" })

      const subTrigger = await findFirstByText(
        ctx.page,
        '[data-slot="context-menu-sub-trigger"]',
        "More Tools"
      )
      await moveMouseTo(subTrigger, ctx.page)
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: context submenu content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, {
        kind: "hover",
        target: "context-menu-sub-trigger:More Tools",
      })

      const subContent = await ctx.page.$('[data-slot="context-menu-sub-content"]')
      if (!subContent) throw new Error("missing context-menu-sub-content after hover")

      const { x: fromX, y: fromY } = await centerOf(subTrigger)
      const { x: toX, y: toY } = await centerOf(subContent)
      const mid = { x: (fromX + toX) / 2, y: (fromY + toY) / 2 }

      await ctx.page.mouse.move(mid.x, mid.y)
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "submenu-grace:mid" })

      await ctx.page.mouse.move(toX, toY)
      await sleep(200)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: context submenu content disappeared: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "hover", target: "submenu-grace:inside" })

      await clickWithinSelectorByText(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        '[role="menuitem"]',
        "Save Page"
      )
      await sleep(50)
      await waitForRole(ctx.page, "menu", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "click", target: "context-menu-sub-item:Save Page" })
    },
  },
  {
    primitive: "context-menu",
    scenario: "submenu-unsafe-leave",
    item: "context-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await rightClickExampleTrigger(
        ctx.page,
        "With Submenu",
        '[data-slot="context-menu-trigger"]'
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "click", target: "context-menu:with-submenu" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="context-menu-sub-trigger"]',
        "More Tools"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "context-menu-sub-trigger:More Tools" })

      const copyItem = await findFirstByText(
        ctx.page,
        '[data-slot="context-menu-item"]',
        "Copy"
      )
      await moveMouseTo(copyItem, ctx.page)
      // Ensure we register a leftwards pointer direction so Radix doesn't treat this as moving
      // towards the submenu.
      const { x: copyX, y: copyY } = await centerOf(copyItem)
      await ctx.page.mouse.move(copyX - 10, copyY)
      await sleep(200)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-sub-content"]',
        false,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-unsafe-leave: context submenu content did not close after leaving: ${String(
            err
          )}`
        )
      })
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="context-menu-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "context-menu-item:Copy" })
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

      await pressChord(ctx.page, ["ArrowDown", "ArrowDown", "Enter"])
      await sleep(50)
      await waitForRole(ctx.page, "listbox", false, Math.min(15000, ctx.timeoutMs))
      await pushStep(ctx, { kind: "press", key: "ArrowDown,ArrowDown,Enter" })
    },
  },
  {
    primitive: "tabs",
    scenario: "click-second-tab",
    item: "tabs-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      const tabs = await ctx.page.$$('[role="tab"]')
      if (tabs.length < 2) throw new Error("expected >=2 role=tab")
      await tabs[1]!.click()
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "tab[1]" })
    },
  },
  {
    primitive: "checkbox",
    scenario: "toggle",
    item: "checkbox-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      await clickFirst(ctx.page, '[role="checkbox"], input[type="checkbox"], button')
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "checkbox" })
    },
  },
  {
    primitive: "switch",
    scenario: "toggle",
    item: "switch-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      await clickFirst(ctx.page, '[role="switch"], button')
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "switch" })
    },
  },
  {
    primitive: "radio-group",
    scenario: "select-second",
    item: "radio-group-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      const radios = await ctx.page.$$('[role="radio"], input[type="radio"]')
      if (radios.length < 2) throw new Error("expected >=2 role=radio")
      await radios[1]!.click()
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "radio[1]" })
    },
  },
  {
    primitive: "slider",
    scenario: "arrow-right",
    item: "slider-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      const slider = await ctx.page.$('[role="slider"], input[type="range"]')
      if (!slider) throw new Error("expected role=slider")
      await slider.focus()
      await sleep(50)
      await pressChord(ctx.page, ["ArrowRight", "ArrowRight"])
      await sleep(50)
      await pushStep(ctx, { kind: "press", key: "ArrowRight,ArrowRight" })
    },
  },
  {
    primitive: "toggle",
    scenario: "toggle",
    item: "toggle-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      await clickFirst(ctx.page, '[aria-pressed], button')
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "toggle" })
    },
  },
  {
    primitive: "toggle-group",
    scenario: "select-second",
    item: "toggle-group-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      const items = await ctx.page.$$('[aria-pressed], [role="radio"], button')
      if (items.length < 2) throw new Error("expected >=2 toggles")
      await items[1]!.click()
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "toggle-group[1]" })
    },
  },
  {
    primitive: "scroll-area",
    scenario: "scroll",
    item: "scroll-area-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      await ctx.page.evaluate(() => {
        const el =
          (document.querySelector("[data-radix-scroll-area-viewport]") as HTMLElement | null) ||
          (document.querySelector('[data-state][style*=\"overflow\"]') as HTMLElement | null)
        if (el) el.scrollTop = 80
      })
      await sleep(50)
      await pushStep(ctx, { kind: "press", key: "scrollTop=80" })
    },
  },
  {
    primitive: "collapsible",
    scenario: "toggle",
    item: "collapsible-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      await clickFirst(ctx.page, '[data-slot="collapsible-trigger"]')
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "collapsible-trigger" })
    },
  },
  {
    primitive: "hover-card",
    scenario: "hover",
    item: "hover-card-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      const trigger =
        (await ctx.page.$("a")) ||
        (await ctx.page.$("button")) ||
        (await ctx.page.$("[data-state]"))
      if (!trigger) throw new Error("missing hover-card trigger")
      await trigger.hover()
      await sleep(200)
      await pushStep(ctx, { kind: "hover", target: "hover-card-trigger" })
      await ctx.page.mouse.move(0, 0)
      await sleep(200)
      await pushStep(ctx, { kind: "hover", target: "mouse-away" })
    },
  },
  {
    primitive: "navigation-menu",
    scenario: "open-close",
    item: "navigation-menu-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      // Radix NavigationMenu isn't a dialog/menu; prefer aria-expanded.
      const trigger =
        (await ctx.page.$('[aria-expanded="false"]')) ||
        (await ctx.page.$("[aria-expanded]")) ||
        (await ctx.page.$("button")) ||
        (await ctx.page.$("a"))
      if (!trigger) throw new Error("missing navigation-menu trigger")
      await trigger.click()
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "navigation-menu-trigger" })
      await press(ctx.page, "Escape")
      await sleep(50)
      await pushStep(ctx, { kind: "press", key: "Escape" })
    },
  },
  {
    primitive: "menubar",
    scenario: "open-navigate-close",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      const items = await ctx.page.$$('[role="menubar"] [role="menuitem"], [role="menuitem"]')
      if (items.length === 0) throw new Error("missing role=menuitem")
      await items[0]!.click()
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "menubar-item[0]" })
      await pressChord(ctx.page, ["ArrowDown", "Escape"])
      await sleep(50)
      await pushStep(ctx, { kind: "press", key: "ArrowDown,Escape" })
    },
  },
  {
    primitive: "menubar",
    scenario: "hover-switch-trigger",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "File"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "menubar:with-submenu:file" })

      await hoverExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "Edit"
      )
      await sleep(200)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "hover", target: "menubar-trigger:Edit" })
    },
  },
  {
    primitive: "menubar",
    scenario: "outside-click-close",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "File"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "menubar:with-submenu:file" })

      // Click outside the menubar example to dismiss.
      await ctx.page.mouse.click(5, 5)
      await sleep(100)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        false,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "outside" })
    },
  },
  {
    primitive: "menubar",
    scenario: "submenu-outside-click-close",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "File"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "menubar:with-submenu:file" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="menubar-sub-trigger"]',
        "Share"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "menubar-sub-trigger:Share" })

      // Click outside the menubar example to dismiss.
      await ctx.page.mouse.click(5, 5)
      await sleep(100)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        false,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "outside" })
    },
  },
  {
    primitive: "menubar",
    scenario: "submenu-keyboard-open-close",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "File"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "menubar:with-submenu:file" })

      const pressed = await pressUntilActiveElementContainsText(
        ctx.page,
        "ArrowDown",
        "Share",
        20
      )
      await press(ctx.page, "ArrowRight")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: [...pressed, "ArrowRight"].join(",") })

      await press(ctx.page, "ArrowLeft")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        false,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: "ArrowLeft" })
    },
  },
  {
    primitive: "menubar",
    scenario: "submenu-arrowleft-escape-close",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "File"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "menubar:with-submenu:file" })

      const pressed = await pressUntilActiveElementContainsText(
        ctx.page,
        "ArrowDown",
        "Share",
        20
      )
      await press(ctx.page, "ArrowRight")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: [...pressed, "ArrowRight"].join(",") })

      await press(ctx.page, "ArrowLeft")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        false,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: "ArrowLeft" })

      await press(ctx.page, "Escape")
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        false,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "press", key: "Escape" })
    },
  },
  {
    primitive: "menubar",
    scenario: "submenu-hover-select",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "File"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      ).catch((err) => {
        throw new Error(
          `submenu-hover-select: menubar content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "click", target: "menubar:with-submenu:file" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="menubar-sub-trigger"]',
        "Share"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-hover-select: menubar submenu content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "hover", target: "menubar-sub-trigger:Share" })

      await clickWithinSelectorByText(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        '[role="menuitem"]',
        "Email link"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        false,
        Math.min(15000, ctx.timeoutMs)
      ).catch((err) => {
        throw new Error(
          `submenu-hover-select: menubar did not close after select: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "click", target: "menubar-sub-item:Email link" })
    },
  },
  {
    primitive: "menubar",
    scenario: "submenu-grace-corridor",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "File"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: menubar content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "click", target: "menubar:with-submenu:file" })

      const subTrigger = await findFirstByText(
        ctx.page,
        '[data-slot="menubar-sub-trigger"]',
        "Share"
      )
      await moveMouseTo(subTrigger, ctx.page)
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: menubar submenu content did not appear: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "hover", target: "menubar-sub-trigger:Share" })

      const subContent = await ctx.page.$('[data-slot="menubar-sub-content"]')
      if (!subContent) throw new Error("missing menubar-sub-content after hover")

      const { x: fromX, y: fromY } = await centerOf(subTrigger)
      const { x: toX, y: toY } = await centerOf(subContent)
      const mid = { x: (fromX + toX) / 2, y: (fromY + toY) / 2 }

      await ctx.page.mouse.move(mid.x, mid.y)
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "submenu-grace:mid" })

      await ctx.page.mouse.move(toX, toY)
      await sleep(200)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        true,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-grace-corridor: menubar submenu content disappeared: ${String(err)}`
        )
      })
      await pushStep(ctx, { kind: "hover", target: "submenu-grace:inside" })

      await clickWithinSelectorByText(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        '[role="menuitem"]',
        "Email link"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        false,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "menubar-sub-item:Email link" })
    },
  },
  {
    primitive: "menubar",
    scenario: "submenu-unsafe-leave",
    item: "menubar-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      await clickExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "File"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        Math.min(15000, ctx.timeoutMs)
      )
      await pushStep(ctx, { kind: "click", target: "menubar:with-submenu:file" })

      await hoverFirstByText(
        ctx.page,
        '[data-slot="menubar-sub-trigger"]',
        "Share"
      )
      await sleep(50)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "menubar-sub-trigger:Share" })

      await hoverExampleWithinSelectorByText(
        ctx.page,
        "With Submenu",
        '[data-slot="menubar-trigger"]',
        "Edit"
      )
      await sleep(200)
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-sub-content"]',
        false,
        ctx.timeoutMs
      ).catch((err) => {
        throw new Error(
          `submenu-unsafe-leave: menubar submenu content did not close after leaving: ${String(
            err
          )}`
        )
      })
      await waitForSelectorPresent(
        ctx.page,
        '[data-slot="menubar-content"]',
        true,
        ctx.timeoutMs
      )
      await pushStep(ctx, { kind: "hover", target: "menubar-trigger:Edit" })
    },
  },
  {
    primitive: "alert-dialog",
    scenario: "open-cancel",
    item: "alert-dialog-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })
      await clickUntilRoleAppears(ctx.page, "button", "alertdialog", ctx.timeoutMs)
      await sleep(50)
      await pushStep(ctx, { kind: "click", target: "alert-dialog-trigger" })
      await press(ctx.page, "Escape")
      await sleep(50)
      await pushStep(ctx, { kind: "press", key: "Escape" })
    },
  },
  {
    primitive: "tooltip",
    scenario: "hover-show-hide",
    item: "tooltip-example",
    async run(ctx) {
      await pushStep(ctx, { kind: "load", url: ctx.url })

      const triggerSelector =
        '[data-slot="tooltip-trigger"], button, a, [aria-describedby], [data-state]'
      await hoverUntilRoleAppears(ctx.page, triggerSelector, "tooltip", ctx.timeoutMs).catch(
        async () => {
          // Some Radix Tooltip variants may not render role=tooltip; fall back to shadcn data-slot.
          await ctx.page.hover('[data-slot="tooltip-trigger"]')
          await waitForSelectorPresent(
            ctx.page,
            '[data-slot="tooltip-content"], [data-slot="tooltip-arrow"]',
            true,
            Math.min(15000, ctx.timeoutMs)
          )
        }
      )
      await sleep(150)
      await pushStep(ctx, { kind: "hover", target: "tooltip-trigger" })

      await press(ctx.page, "Escape")
      await sleep(150)
      await waitForRole(ctx.page, "tooltip", false, Math.min(15000, ctx.timeoutMs)).catch(
        async () => {
          await waitForSelectorPresent(
            ctx.page,
            '[data-slot="tooltip-content"], [data-slot="tooltip-arrow"]',
            false,
            Math.min(15000, ctx.timeoutMs)
          )
        }
      )
      await pushStep(ctx, { kind: "press", key: "Escape" })
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

if (process.env.DEBUG_ARGS === "1") {
  console.log("?? argv debug")
  console.log({ flags, names })
}

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

const theme: Theme =
  ((typeof flags.theme === "string" ? flags.theme : undefined) ??
    (process.env.THEME as Theme | undefined) ??
    "light") === "dark"
    ? "dark"
    : "light"

function selectedScenarios(): Scenario[] {
  if (!all) {
    const wanted = new Set(names)
    if (wanted.size === 0) {
      return scenarios.filter((s) =>
        [
          "dialog-example",
          "popover-example",
          "dropdown-menu-example",
          "select-example",
          "tabs-example",
          "tooltip-example",
        ].includes(s.item)
      )
    }
    return scenarios.filter((s) => {
      if (wanted.has(s.item) || wanted.has(s.primitive)) return true
      const full = `${s.item}.${s.primitive}.${s.scenario}`
      if (wanted.has(full)) return true
      const short = `${s.primitive}.${s.scenario}`
      if (wanted.has(short)) return true
      return false
    })
  }
  return scenarios
}

async function loadDefaultDesignSystemConfig(repoRoot: string) {
  const configPath = path.join(
    repoRoot,
    "repo-ref",
    "ui",
    "apps",
    "v4",
    "registry",
    "config.ts"
  )
  if (!fs.existsSync(configPath)) return null
  try {
    const mod = await import(pathToFileURL(configPath).href)
    return (mod as any).DEFAULT_CONFIG ?? null
  } catch {
    return null
  }
}

async function main() {
  const puppeteer = await loadPuppeteer()
  ensureDir(outDir)
  const executablePath = resolveBrowserExecutablePath()

  const upstreamDefault = await loadDefaultDesignSystemConfig(repoRoot)
  const defaultConfig =
    upstreamDefault && typeof upstreamDefault === "object"
      ? upstreamDefault
      : {
          base: "radix",
          style: "vega",
          baseColor: "neutral",
          theme: "neutral",
          radius: "default",
          font: "inter",
          iconLibrary: "lucide",
          menuAccent: "subtle",
          menuColor: "default",
        }

  const config = {
    base: "radix",
    style:
      (typeof flags.style === "string" ? flags.style : undefined) ??
      process.env.STYLE ??
      defaultConfig.style ??
      "vega",
    baseColor:
      (typeof flags.baseColor === "string" ? flags.baseColor : undefined) ??
      process.env.BASE_COLOR ??
      defaultConfig.baseColor ??
      "neutral",
    theme:
      (typeof flags.dsTheme === "string" ? flags.dsTheme : undefined) ??
      process.env.DS_THEME ??
      defaultConfig.theme ??
      "neutral",
    radius:
      (typeof flags.radius === "string" ? flags.radius : undefined) ??
      process.env.RADIUS ??
      defaultConfig.radius ??
      "default",
    font:
      (typeof flags.font === "string" ? flags.font : undefined) ??
      process.env.FONT ??
      defaultConfig.font ??
      "inter",
    iconLibrary:
      (typeof flags.iconLibrary === "string" ? flags.iconLibrary : undefined) ??
      process.env.ICON_LIBRARY ??
      defaultConfig.iconLibrary ??
      "lucide",
    menuAccent:
      (typeof flags.menuAccent === "string" ? flags.menuAccent : undefined) ??
      process.env.MENU_ACCENT ??
      defaultConfig.menuAccent ??
      "subtle",
    menuColor:
      (typeof flags.menuColor === "string" ? flags.menuColor : undefined) ??
      process.env.MENU_COLOR ??
      defaultConfig.menuColor ??
      "default",
  }

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

      // Behavior goldens should be deterministic. External images add timing and network
      // nondeterminism (e.g. Radix Avatar conditionally mounting <img/> only once loaded), so we
      // abort image requests and rely on fallbacks.
      await page.setRequestInterception(true)
      page.on("request", (req) => {
        if (req.resourceType() === "image") {
          void req.abort()
          return
        }
        void req.continue()
      })

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
