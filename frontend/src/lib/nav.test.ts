import { describe, expect, it } from 'vitest'
import { nav } from './nav'

describe('nav', () => {
  it('has all 9 spec routes', () => {
    expect(nav.map((n) => n.label)).toEqual([
      'Dashboard',
      'Nodes',
      'Resellers',
      'Direct Users',
      'Services',
      'Notifications',
      'Audit',
      'Settings',
    ])
  })

  it('has unique paths', () => {
    const paths = nav.map((n) => n.to)
    expect(new Set(paths).size).toBe(paths.length)
  })
})
