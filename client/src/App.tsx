import { useCallback, useState, type FormEvent } from 'react'
import './App.css'

type SubmitResponse = {
  ai_status: string
  disbursement: {
    mode: string
    tx_hash?: string
    detail?: string
  }
  idempotency_replay?: boolean
}

function App() {
  const [collector, setCollector] = useState('')
  const [file, setFile] = useState<File | null>(null)
  const [idempotencyKey, setIdempotencyKey] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<SubmitResponse | null>(null)

  const onSubmit = useCallback(
    async (e: FormEvent) => {
      e.preventDefault()
      setError(null)
      setResult(null)
      if (!collector.trim()) {
        setError('Enter your Stellar public address (G…).')
        return
      }
      if (!file) {
        setError('Choose a photo of your collected bag.')
        return
      }
      setLoading(true)
      try {
        const body = new FormData()
        body.append('collector_g_address', collector.trim())
        body.append('image', file)
        if (idempotencyKey.trim()) {
          body.append('idempotency_key', idempotencyKey.trim())
        }
        const headers: HeadersInit = {}
        if (idempotencyKey.trim()) {
          headers['Idempotency-Key'] = idempotencyKey.trim()
        }
        const res = await fetch('/api/submit', { method: 'POST', body, headers })
        const data = await res.json().catch(() => ({}))
        if (!res.ok) {
          setError(typeof data.error === 'string' ? data.error : res.statusText)
          return
        }
        setResult(data as SubmitResponse)
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Network error')
      } finally {
        setLoading(false)
      }
    },
    [collector, file, idempotencyKey],
  )

  return (
    <div className="rw-app">
      <header className="rw-header">
        <h1>River Warrior</h1>
        <p className="rw-tagline">
          Verified cleanup bounty — AI check, Soroban USDC payout on Stellar.
        </p>
      </header>

      <main className="rw-main">
        <form className="rw-card" onSubmit={onSubmit}>
          <label className="rw-field">
            <span>Stellar address</span>
            <input
              type="text"
              name="collector"
              autoComplete="off"
              placeholder="G… (56 characters)"
              value={collector}
              onChange={(e) => setCollector(e.target.value)}
              inputMode="text"
            />
          </label>

          <label className="rw-field">
            <span>Photo of collected trash</span>
            <input
              type="file"
              accept="image/*"
              capture="environment"
              onChange={(e) => setFile(e.target.files?.[0] ?? null)}
            />
            {file ? <small className="rw-file-name">{file.name}</small> : null}
          </label>

          <label className="rw-field">
            <span>Idempotency key (optional)</span>
            <input
              type="text"
              placeholder="UUID — prevents double pay on retry"
              value={idempotencyKey}
              onChange={(e) => setIdempotencyKey(e.target.value)}
            />
          </label>

          <button type="submit" className="rw-submit" disabled={loading}>
            {loading ? 'Submitting…' : 'Submit for verification'}
          </button>
        </form>

        {error ? (
          <div className="rw-banner rw-banner-error" role="alert">
            {error}
          </div>
        ) : null}

        {result ? (
          <section className="rw-card rw-result" aria-live="polite">
            <h2>Result</h2>
            <p>
              <strong>AI:</strong>{' '}
              <span
                className={
                  result.ai_status === 'VERIFIED' ? 'rw-pill rw-pill-ok' : 'rw-pill rw-pill-bad'
                }
              >
                {result.ai_status}
              </span>
            </p>
            <p>
              <strong>Disbursement:</strong> {result.disbursement.mode}
              {result.disbursement.tx_hash ? (
                <>
                  {' '}
                  <code className="rw-hash">{result.disbursement.tx_hash}</code>
                </>
              ) : null}
            </p>
            {result.disbursement.detail ? (
              <p className="rw-detail">{result.disbursement.detail}</p>
            ) : null}
            {result.idempotency_replay ? (
              <p className="rw-detail">Replayed prior submission (idempotent).</p>
            ) : null}
          </section>
        ) : null}
      </main>

      <footer className="rw-footer">
        <p>
          Run the API on port <code>8787</code> and <code>npm run dev</code> — requests proxy to
          the backend. Set <code>OPENAI_API_KEY</code> for real vision; otherwise AI defaults to
          VERIFIED in development.
        </p>
      </footer>
    </div>
  )
}

export default App
