import type { SemanticsNodeModel, SemanticsModel, Selector, ScriptStep } from './types'

export function bestSelector(node: SemanticsNodeModel): Selector {
  if (node.testId) {
    return { kind: 'test_id', id: node.testId }
  }

  const name = node.label ?? node.name
  if (node.role && name) {
    return {
      kind: 'role_and_name',
      role: node.role.toLowerCase().replace(/\s+/g, '_'),
      name,
    }
  }

  return { kind: 'node_id', id: node.id }
}

export function selectorToJson(selector: Selector): string {
  return JSON.stringify(selector, null, 2)
}

export function nodePath(node: SemanticsNodeModel, semantics: SemanticsModel): string {
  const path: string[] = []
  let current: SemanticsNodeModel | undefined = node

  while (current) {
    const identifier = current.testId ?? current.role ?? current.id
    path.unshift(identifier)
    current = current.parentId ? semantics.nodesById[current.parentId] : undefined
  }

  return path.join(' > ')
}

export function generateScriptStep(node: SemanticsNodeModel, action: string = 'click'): ScriptStep {
  return {
    type: action,
    target: bestSelector(node),
  }
}

export function scriptStepToJson(step: ScriptStep): string {
  return JSON.stringify(step, null, 2)
}

export function copyToClipboard(text: string): Promise<void> {
  return navigator.clipboard.writeText(text)
}
