'use client';

import { useEffect } from 'react';

const STORAGE_KEY = 'yl_lock_expiries';
const SEVEN_DAYS = 7 * 24 * 60 * 60 * 1000;

export interface LockEntry {
  tier: string;
  lockUntil: number; // epoch ms
}

/** Store lock expiries in localStorage for background notification checks. */
export function storeLockExpiries(entries: LockEntry[]) {
  if (typeof window === 'undefined') return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
}

/**
 * On mount, checks localStorage for locks expiring within 7 days
 * and fires a browser Notification for each one (if permission is granted).
 */
export function useLockExpiryNotifications() {
  useEffect(() => {
    if (typeof window === 'undefined' || !('Notification' in window)) return;
    if (Notification.permission !== 'granted') return;

    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return;

    let entries: LockEntry[];
    try {
      entries = JSON.parse(raw);
    } catch {
      return;
    }

    const now = Date.now();
    for (const { tier, lockUntil } of entries) {
      const diff = lockUntil - now;
      if (diff > 0 && diff <= SEVEN_DAYS) {
        const days = Math.ceil(diff / 86_400_000);
        new Notification('YieldLadder — Lock Expiring Soon', {
          body: `Your ${tier} vault unlocks in ${days} day${days !== 1 ? 's' : ''}. Visit YieldLadder to withdraw.`,
          icon: '/favicon.ico',
        });
      }
    }
  }, []);
}

/** Request browser notification permission and return the result. */
export async function requestNotificationPermission(): Promise<NotificationPermission> {
  if (typeof window === 'undefined' || !('Notification' in window)) return 'denied';
  return Notification.requestPermission();
}
