'use client'

import { useState, useMemo, useCallback } from 'react'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Input } from '@/components/ui/input'
import { cn } from '@/lib/utils'
import { useTranslation } from '@/hooks/use-i18n'
import { ChevronRight, ChevronDown, Search } from 'lucide-react'

interface JsonNodeProps {
  keyName?: string
  value: unknown
  depth: number
  searchQuery: string
  defaultExpanded?: boolean
}

function JsonNode({ keyName, value, depth, searchQuery, defaultExpanded = false }: JsonNodeProps) {
  const { t } = useTranslation()
  const [isExpanded, setIsExpanded] = useState(defaultExpanded || depth < 2)

  const isObject = value !== null && typeof value === 'object'
  const isArray = Array.isArray(value)

  const matches = useMemo(() => {
    if (!searchQuery) return false
    const query = searchQuery.toLowerCase()
    if (keyName?.toLowerCase().includes(query)) return true
    if (!isObject && String(value).toLowerCase().includes(query)) return true
    return false
  }, [keyName, value, isObject, searchQuery])

  const renderValue = useCallback(() => {
    if (value === null) return <span className="text-muted-foreground">{t('json.null')}</span>
    if (value === undefined) return <span className="text-muted-foreground">{t('json.undefined')}</span>
    if (typeof value === 'boolean') return <span className="text-chart-5">{String(value)}</span>
    if (typeof value === 'number') return <span className="text-chart-3">{value}</span>
    if (typeof value === 'string') return <span className="text-chart-2">{'"'}{value}{'"'}</span>
    return null
  }, [t, value])

  if (!isObject) {
    return (
      <div
        className={cn(
          'flex items-center gap-1 py-0.5 font-mono text-xs',
          matches && 'bg-chart-4/20 rounded'
        )}
        style={{ paddingLeft: `${depth * 16}px` }}
      >
        {keyName && (
          <>
            <span className="text-chart-1">{'"'}{keyName}{'"'}</span>
            <span className="text-muted-foreground">:</span>
          </>
        )}
        {renderValue()}
      </div>
    )
  }

  const entries = isArray
    ? (value as unknown[]).map((v, i) => [String(i), v] as const)
    : Object.entries(value as Record<string, unknown>)

  const preview = isArray
    ? `[${entries.length}]`
    : `{${entries.length}}`

  return (
    <div>
      <div
        className={cn(
          'flex items-center gap-1 py-0.5 font-mono text-xs cursor-pointer hover:bg-accent rounded',
          matches && 'bg-chart-4/20'
        )}
        style={{ paddingLeft: `${depth * 16}px` }}
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <span className="w-4 h-4 flex items-center justify-center text-muted-foreground">
          {isExpanded ? <ChevronDown className="h-3 w-3" /> : <ChevronRight className="h-3 w-3" />}
        </span>
        {keyName && (
          <>
            <span className="text-chart-1">{'"'}{keyName}{'"'}</span>
            <span className="text-muted-foreground">:</span>
          </>
        )}
        <span className="text-muted-foreground">{preview}</span>
      </div>
      {isExpanded && (
        <div>
          {entries.map(([k, v]) => (
            <JsonNode
              key={k}
              keyName={isArray ? undefined : k}
              value={v}
              depth={depth + 1}
              searchQuery={searchQuery}
            />
          ))}
        </div>
      )}
    </div>
  )
}

interface RawJsonViewProps {
  data: unknown
  title?: string
}

export function RawJsonView({ data, title }: RawJsonViewProps) {
  const [searchQuery, setSearchQuery] = useState('')

  return (
    <div className="flex flex-col h-full">
      {title && (
        <div className="px-3 py-2 border-b border-border">
          <h3 className="text-sm font-medium text-foreground">{title}</h3>
        </div>
      )}
      <div className="px-3 py-2 border-b border-border">
        <div className="relative">
          <Search className="absolute left-2 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="Search JSON..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-7 h-7 text-xs"
          />
        </div>
      </div>
      <ScrollArea className="flex-1">
        <div className="p-2">
          <JsonNode value={data} depth={0} searchQuery={searchQuery} defaultExpanded />
        </div>
      </ScrollArea>
    </div>
  )
}
