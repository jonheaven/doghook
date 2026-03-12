import { StrictMode, useMemo, useState } from 'react';
import { createRoot } from 'react-dom/client';
import {
  ConnectWalletButton,
  DogestashProvider,
  useUnifiedWallet,
} from '@jonheaven/dogestash';

const styles = `
  .kabosu-wallet-shell {
    color: #f8f4ea;
    font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }

  .kabosu-wallet-heading {
    margin: 0;
    color: #fff4d6;
    font-size: 1.55rem;
    font-weight: 700;
  }

  .kabosu-wallet-kicker {
    margin: 0 0 0.65rem;
    color: rgba(253, 230, 138, 0.7);
    font-size: 0.68rem;
    font-weight: 700;
    letter-spacing: 0.32em;
    text-transform: uppercase;
  }

  .kabosu-wallet-copy {
    margin: 0.75rem 0 0;
    color: rgba(255, 255, 255, 0.62);
    font-size: 0.92rem;
    line-height: 1.55;
  }

  .kabosu-wallet-card {
    margin-top: 1.1rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.35rem;
    background: rgba(255, 255, 255, 0.04);
    padding: 1rem;
  }

  .kabosu-wallet-connect {
    width: 100%;
    border: 1px solid rgba(251, 191, 36, 0.58);
    border-radius: 0.95rem;
    background: linear-gradient(135deg, rgba(251, 191, 36, 0.92), rgba(253, 224, 71, 0.88));
    color: #1b1204;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 700;
    padding: 0.95rem 1rem;
    transition: transform 160ms ease, box-shadow 160ms ease, opacity 160ms ease;
    box-shadow: 0 18px 40px rgba(0, 0, 0, 0.24);
  }

  .kabosu-wallet-connect:hover {
    transform: translateY(-1px);
    box-shadow: 0 22px 46px rgba(0, 0, 0, 0.28);
  }

  .kabosu-wallet-grid {
    display: grid;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .kabosu-wallet-pane {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1rem;
    background: rgba(0, 0, 0, 0.28);
    padding: 0.9rem;
  }

  .kabosu-wallet-row {
    align-items: center;
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .kabosu-wallet-label {
    color: rgba(255, 255, 255, 0.45);
    font-size: 0.8rem;
  }

  .kabosu-wallet-value {
    color: #f7d774;
    font-size: 0.84rem;
    font-weight: 600;
  }

  .kabosu-wallet-mono {
    word-break: break-all;
    margin: 0.55rem 0 0;
    color: rgba(255, 255, 255, 0.82);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
    font-size: 0.78rem;
    line-height: 1.55;
  }

  .kabosu-wallet-balance {
    margin: 0.55rem 0 0;
    color: #fff4d6;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
    font-size: 1.12rem;
    font-weight: 700;
  }

  .kabosu-wallet-actions {
    display: grid;
    gap: 1rem;
    margin-top: 1rem;
  }

  .kabosu-wallet-title {
    margin: 0;
    color: #fff4d6;
    font-size: 0.96rem;
    font-weight: 700;
  }

  .kabosu-wallet-hint {
    color: rgba(255, 255, 255, 0.42);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.16em;
  }

  .kabosu-wallet-textarea,
  .kabosu-wallet-input {
    width: 100%;
    box-sizing: border-box;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.95rem;
    background: rgba(0, 0, 0, 0.36);
    color: #fff4d6;
    font: inherit;
    margin-top: 0.8rem;
    outline: none;
    padding: 0.85rem 0.95rem;
  }

  .kabosu-wallet-textarea:focus,
  .kabosu-wallet-input:focus {
    border-color: rgba(251, 191, 36, 0.68);
  }

  .kabosu-wallet-field-grid {
    display: grid;
    gap: 0.8rem;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    margin-top: 0.85rem;
  }

  .kabosu-wallet-field-label {
    color: rgba(255, 255, 255, 0.45);
    display: grid;
    font-size: 0.72rem;
    gap: 0.38rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .kabosu-wallet-button {
    width: 100%;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.95rem;
    background: rgba(29, 22, 16, 0.95);
    color: #f7d774;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 700;
    margin-top: 0.8rem;
    padding: 0.85rem 1rem;
    transition: opacity 160ms ease, transform 160ms ease, border-color 160ms ease;
  }

  .kabosu-wallet-button:hover:not(:disabled) {
    border-color: rgba(251, 191, 36, 0.58);
    transform: translateY(-1px);
  }

  .kabosu-wallet-button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .kabosu-wallet-buttonRow {
    display: grid;
    gap: 0.7rem;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    margin-top: 0.85rem;
  }

  .kabosu-wallet-button--emerald {
    background: rgba(52, 211, 153, 0.12);
    border-color: rgba(52, 211, 153, 0.2);
    color: #d1fae5;
    margin-top: 0;
  }

  .kabosu-wallet-button--sky {
    background: rgba(56, 189, 248, 0.12);
    border-color: rgba(56, 189, 248, 0.2);
    color: #dbeafe;
    margin-top: 0;
  }

  .kabosu-wallet-error {
    margin-top: 1rem;
    border: 1px solid rgba(248, 113, 113, 0.22);
    border-radius: 1rem;
    background: rgba(239, 68, 68, 0.12);
    color: #fecaca;
    font-size: 0.88rem;
    padding: 0.9rem 1rem;
  }

  .kabosu-wallet-result {
    margin-top: 1rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.05rem;
    background: rgba(0, 0, 0, 0.32);
    padding: 1rem;
  }

  .kabosu-wallet-result pre {
    margin: 0.7rem 0 0;
    min-height: 11rem;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 0.9rem;
    background: rgba(0, 0, 0, 0.42);
    color: #f7d774;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
    font-size: 0.74rem;
    line-height: 1.55;
    padding: 0.9rem;
  }

  @media (max-width: 640px) {
    .kabosu-wallet-field-grid,
    .kabosu-wallet-buttonRow {
      grid-template-columns: minmax(0, 1fr);
    }
  }
`;

function formatBalance(balance: number, verified: boolean) {
  if (!verified) return 'Balance pending';
  return `${balance.toFixed(8)} DOGE`;
}

function parseInteger(value: string, fallback: number) {
  const parsed = Number.parseInt(value, 10);
  return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : fallback;
}

function WalletTools() {
  const {
    connected,
    address,
    balance,
    balanceVerified,
    signMessage,
    signDMPIntent,
    walletType,
  } = useUnifiedWallet();
  const [message, setMessage] = useState('Kabosu monitor heartbeat: additive viewing is live.');
  const [priceKoinu, setPriceKoinu] = useState('4206900000');
  const [expiryHeight, setExpiryHeight] = useState('6000000');
  const [result, setResult] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [activeAction, setActiveAction] = useState<'message' | 'listing' | 'bid' | null>(null);

  const connectionLabel = useMemo(() => walletType ?? 'not connected', [walletType]);

  const resetState = () => {
    setError(null);
    setResult('');
  };

  const showError = (messageText: string) => {
    setError(messageText);
    if (typeof window !== 'undefined' && typeof window.showToast === 'function') {
      window.showToast(messageText, 'error');
    }
  };

  const handleSignMessage = async () => {
    resetState();
    setActiveAction('message');

    try {
      const signature = await signMessage(message);
      setResult(signature);
      window.showToast?.('Signed monitor message.');
    } catch (nextError) {
      showError(nextError instanceof Error ? nextError.message : 'Message signing failed');
    } finally {
      setActiveAction(null);
    }
  };

  const handleSignListing = async () => {
    resetState();
    setActiveAction('listing');

    try {
      const signed = await signDMPIntent('listing', {
        price_koinu: parseInteger(priceKoinu, 4206900000),
        psbt_cid: 'ipfs://QmKabosuMonitorListing',
        expiry_height: parseInteger(expiryHeight, 6000000),
      });
      setResult(JSON.stringify(signed, null, 2));
      window.showToast?.('Signed DMP listing intent.');
    } catch (nextError) {
      showError(nextError instanceof Error ? nextError.message : 'Listing signing failed');
    } finally {
      setActiveAction(null);
    }
  };

  const handleSignBid = async () => {
    resetState();
    setActiveAction('bid');

    try {
      const signed = await signDMPIntent('bid', {
        listing_id: `${'b'.repeat(64)}i0`,
        price_koinu: parseInteger(priceKoinu, 4206900000),
        psbt_cid: 'ipfs://QmKabosuMonitorBid',
        expiry_height: parseInteger(expiryHeight, 6000000),
      });
      setResult(JSON.stringify(signed, null, 2));
      window.showToast?.('Signed DMP bid intent.');
    } catch (nextError) {
      showError(nextError instanceof Error ? nextError.message : 'Bid signing failed');
    } finally {
      setActiveAction(null);
    }
  };

  return (
    <div className="kabosu-wallet-shell">
      <style>{styles}</style>

      <p className="kabosu-wallet-kicker">Wallet Tools</p>
      <h2 className="kabosu-wallet-heading">Dogestash control deck</h2>
      <p className="kabosu-wallet-copy">
        Connect a wallet, verify balance context, sign test messages, and generate DMP
        intents directly from the live monitor.
      </p>

      <div className="kabosu-wallet-card">
        <ConnectWalletButton
          className="kabosu-wallet-connect"
          connectLabel="Connect Wallet"
          disconnectLabel="Disconnect"
        />

        <div className="kabosu-wallet-grid">
          <div className="kabosu-wallet-pane">
            <div className="kabosu-wallet-row">
              <span className="kabosu-wallet-label">Wallet</span>
              <span className="kabosu-wallet-value">{connectionLabel}</span>
            </div>
            <p className="kabosu-wallet-mono">
              {address ?? 'Connect any supported wallet to activate monitor tools.'}
            </p>
          </div>

          <div className="kabosu-wallet-pane">
            <div className="kabosu-wallet-row">
              <span className="kabosu-wallet-label">Balance check</span>
              <span className="kabosu-wallet-value">
                {balanceVerified ? 'verified' : 'awaiting provider'}
              </span>
            </div>
            <p className="kabosu-wallet-balance">{formatBalance(balance, balanceVerified)}</p>
          </div>
        </div>
      </div>

      <div className="kabosu-wallet-actions">
        <div className="kabosu-wallet-card">
          <div className="kabosu-wallet-row">
            <h3 className="kabosu-wallet-title">Sign message</h3>
            <span className="kabosu-wallet-hint">CLI parity</span>
          </div>
          <textarea
            className="kabosu-wallet-textarea"
            rows={4}
            value={message}
            onChange={(event) => setMessage(event.target.value)}
            placeholder="Sign a monitor status message"
          />
          <button
            className="kabosu-wallet-button"
            disabled={!connected || activeAction !== null}
            onClick={handleSignMessage}
            type="button"
          >
            {activeAction === 'message' ? 'Signing...' : 'Sign Message'}
          </button>
        </div>

        <div className="kabosu-wallet-card">
          <div className="kabosu-wallet-row">
            <h3 className="kabosu-wallet-title">DMP test intents</h3>
            <span className="kabosu-wallet-hint">listing + bid</span>
          </div>
          <div className="kabosu-wallet-field-grid">
            <label className="kabosu-wallet-field-label">
              price_koinu
              <input
                className="kabosu-wallet-input"
                value={priceKoinu}
                onChange={(event) => setPriceKoinu(event.target.value)}
              />
            </label>
            <label className="kabosu-wallet-field-label">
              expiry_height
              <input
                className="kabosu-wallet-input"
                value={expiryHeight}
                onChange={(event) => setExpiryHeight(event.target.value)}
              />
            </label>
          </div>
          <div className="kabosu-wallet-buttonRow">
            <button
              className="kabosu-wallet-button kabosu-wallet-button--emerald"
              disabled={!connected || activeAction !== null}
              onClick={handleSignListing}
              type="button"
            >
              {activeAction === 'listing' ? 'Signing listing...' : 'Sign Listing'}
            </button>
            <button
              className="kabosu-wallet-button kabosu-wallet-button--sky"
              disabled={!connected || activeAction !== null}
              onClick={handleSignBid}
              type="button"
            >
              {activeAction === 'bid' ? 'Signing bid...' : 'Sign Bid'}
            </button>
          </div>
        </div>
      </div>

      {error ? <div className="kabosu-wallet-error">{error}</div> : null}

      <div className="kabosu-wallet-result">
        <div className="kabosu-wallet-row">
          <h3 className="kabosu-wallet-title">Result</h3>
          <span className="kabosu-wallet-hint">
            {connected ? 'live wallet output' : 'connect to test'}
          </span>
        </div>
        <pre>{result || 'Signed messages and DMP payloads will appear here.'}</pre>
      </div>
    </div>
  );
}

function App() {
  return (
    <StrictMode>
      <DogestashProvider>
        <WalletTools />
      </DogestashProvider>
    </StrictMode>
  );
}

const mountNode = document.getElementById('wallet-tools-root');

if (mountNode) {
  createRoot(mountNode).render(<App />);
}
