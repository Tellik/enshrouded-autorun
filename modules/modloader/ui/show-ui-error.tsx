import { post } from '../../transport/ui/send-host-command'
import React, { Component } from 'react'
import type { ErrorInfo } from 'react'

export class UiErrorBoundary extends Component<
  { children: React.ReactNode },
  { hasError: boolean }
> {
  state = { hasError: false }
  static getDerivedStateFromError() {
    return { hasError: true }
  }
  componentDidCatch(error: Error, info: ErrorInfo) {
    post('ui-error', {
      message: `React render failed: ${error.stack || error.message}\n${info.componentStack || ''}`,
    })
  }
  render() {
    return this.state.hasError ? (
      <main className="ui-fatal-error">
        <h1>ShroudForge could not display this view</h1>
        <p>
          The error was recorded in shroudforge-client.log. Restart the
          Modloader after fixing the reported cause.
        </p>
      </main>
    ) : (
      this.props.children
    )
  }
}
