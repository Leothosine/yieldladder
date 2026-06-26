'use client';

import { useState, useEffect } from 'react';

export interface AssetPrices {
  XLM: number;
  EURC: number;
  AQUA: number;
  updatedAt: number;
}

const HORIZON = 'https://horizon.stellar.org';
const CACHE_TTL_MS = 60_000;

// USDC issuer on Stellar mainnet
const USDC_ISSUER = 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN';
// EURC issuer on Stellar mainnet
const EURC_ISSUER = 'GDHU6WRG4IEQXM5NZ4BMPKOXHW76MZM4Y2IEMFDVXBSDP6SJY4ITNPP';

interface OrderBook {
  bids: { price: string }[];
}

async function fetchBestBid(
  sellingType: string,
  buyingCode: string,
  buyingIssuer: string,
): Promise<number> {
  try {
    const params = new URLSearchParams({
      selling_asset_type: sellingType,
      buying_asset_type: 'credit_alphanum4',
      buying_asset_code: buyingCode,
      buying_asset_issuer: buyingIssuer,
      limit: '1',
    });
    const res = await fetch(`${HORIZON}/order_book?${params.toString()}`);
    if (!res.ok) return 0;
    const book = (await res.json()) as OrderBook;
    return parseFloat(book.bids?.[0]?.price ?? '0');
  } catch {
    return 0;
  }
}

let _cache: AssetPrices | null = null;
let _cacheTime = 0;

export async function fetchAssetPrices(): Promise<AssetPrices> {
  const now = Date.now();
  if (_cache && now - _cacheTime < CACHE_TTL_MS) return _cache;

  const [xlm, eurc] = await Promise.all([
    fetchBestBid('native', 'USDC', USDC_ISSUER),
    fetchBestBid('native', 'EURC', EURC_ISSUER),
  ]);

  _cache = { XLM: xlm, EURC: eurc, AQUA: 0, updatedAt: now };
  _cacheTime = now;
  return _cache;
}

export function useAssetPrices() {
  const [prices, setPrices] = useState<AssetPrices | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const p = await fetchAssetPrices();
        if (!cancelled) {
          setPrices(p);
          setLoading(false);
        }
      } catch (e) {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : String(e));
          setLoading(false);
        }
      }
    }

    load();
    const interval = setInterval(load, CACHE_TTL_MS);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  return { prices, loading, error };
}
