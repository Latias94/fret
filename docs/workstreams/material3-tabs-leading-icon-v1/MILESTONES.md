# Material3 Tabs Leading Icon v1 - Milestones

Status: Closed
Last updated: 2026-05-30

## M0 - Source and Scope

Complete. The lane targets Compose `LeadingIconTab`, not the taller generic stacked icon + text
`Tab` path.

## M1 - Implementation

Complete. `TabItem::leading_icon(IconId)` renders a 24px leading icon, an 8px icon-label gap, and
token-routed icon colors for primary and secondary tabs.

## M2 - Evidence

Complete. Focused tabs tests and v30 token tests cover the shipped API and token aliases.
