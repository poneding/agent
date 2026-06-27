import type {
  DesktopBootstrapDiagnostic,
  DesktopConfigSnapshot,
  DesktopSettings,
} from '../types';

export function SettingsView({
  snapshot,
  settings,
  diagnostics,
  error,
}: {
  snapshot: DesktopConfigSnapshot | null;
  settings: DesktopSettings;
  diagnostics: DesktopBootstrapDiagnostic[];
  error: string | null;
}) {
  if (error !== null) {
    return (
      <section className="diagnostic diagnostic-error" role="alert">
        <strong>Settings unavailable</strong>
        <span>{error}</span>
      </section>
    );
  }

  return (
    <div className="settings-view">
      {diagnostics.length > 0 ? (
        <section className="diagnostic diagnostic-error" role="alert">
          <strong>{diagnostics[0].code}</strong>
          <span>{diagnostics[0].message}</span>
        </section>
      ) : null}

      <section className="settings-section">
        <h2>Model</h2>
        <dl>
          <dt>Provider</dt>
          <dd>{snapshot?.snapshot.selected_provider ?? settings.model.provider}</dd>
          <dt>Model</dt>
          <dd>{snapshot?.snapshot.selected_model ?? settings.model.model}</dd>
        </dl>
      </section>

      <section className="settings-section">
        <h2>Policy</h2>
        <dl>
          <dt>Default</dt>
          <dd>{settings.policy.default_decision}</dd>
          <dt>Shell approval</dt>
          <dd>{settings.policy.require_approval_for_shell ? 'required' : 'not required'}</dd>
        </dl>
      </section>

      <section className="settings-section">
        <h2>Memory</h2>
        <dl>
          <dt>Project</dt>
          <dd>{settings.memory.project_memory_enabled ? 'enabled' : 'disabled'}</dd>
          <dt>Long term</dt>
          <dd>{settings.memory.long_term_memory_enabled ? 'enabled' : 'disabled'}</dd>
        </dl>
      </section>

      <section className="settings-section">
        <h2>MCP</h2>
        <dl>
          <dt>Status</dt>
          <dd>{settings.mcp.enabled ? 'enabled' : 'disabled'}</dd>
          <dt>Servers</dt>
          <dd>{settings.mcp.server_count}</dd>
        </dl>
      </section>

      <section className="settings-section">
        <h2>Interface</h2>
        <dl>
          <dt>Theme</dt>
          <dd>{settings.ui.theme}</dd>
          <dt>Trace</dt>
          <dd>{settings.ui.show_trace_timeline ? 'shown' : 'hidden'}</dd>
        </dl>
      </section>

      <section className="settings-section">
        <h2>Providers</h2>
        {snapshot === null || snapshot.snapshot.providers.length === 0 ? (
          <div className="empty-list">No providers</div>
        ) : (
          <ul className="provider-list" aria-label="Model providers">
            {snapshot.snapshot.providers.map((provider) => (
              <li key={provider.id}>
                <span>{provider.id}</span>
                <small>
                  {provider.kind} / {provider.default_model} /{' '}
                  {provider.enabled ? 'enabled' : 'disabled'}
                </small>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="settings-section">
        <h2>Config Diagnostics</h2>
        {snapshot === null || snapshot.snapshot.diagnostics.length === 0 ? (
          <p>No diagnostics</p>
        ) : (
          <ul>
            {snapshot.snapshot.diagnostics.map((diagnostic) => (
              <li key={`${diagnostic.code}-${diagnostic.message}`}>
                <strong>{diagnostic.code}</strong>
                <span>{diagnostic.message}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
