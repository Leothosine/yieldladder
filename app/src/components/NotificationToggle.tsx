'use client';

import { useState, useEffect } from 'react';
import { requestNotificationPermission, storeLockExpiries, type LockEntry } from '../lib/notifications';

interface Props {
  locks: LockEntry[];
}

export function NotificationToggle({ locks }: Props) {
  const [permission, setPermission] = useState<NotificationPermission>('default');

  useEffect(() => {
    if ('Notification' in window) setPermission(Notification.permission);
  }, []);

  if (!('Notification' in (typeof window !== 'undefined' ? window : {}))) return null;
  if (permission === 'denied') return null;

  async function enable() {
    const result = await requestNotificationPermission();
    setPermission(result);
    if (result === 'granted') storeLockExpiries(locks);
  }

  if (permission === 'granted') {
    return (
      <p style={{ fontSize: '0.8rem', color: '#16a34a', margin: 0 }}>
        🔔 Lock expiry notifications enabled
      </p>
    );
  }

  return (
    <button
      onClick={enable}
      style={{
        background: 'none',
        border: '1px solid #e2e8f0',
        borderRadius: 6,
        padding: '0.35rem 0.75rem',
        fontSize: '0.8rem',
        cursor: 'pointer',
        color: '#475569',
      }}
    >
      🔔 Enable lock expiry notifications
    </button>
  );
}
