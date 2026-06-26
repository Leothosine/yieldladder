'use client';

import type { HarvestEvent } from './mockData';

interface YieldChartProps {
  data: HarvestEvent[];
  tier: string;
}

const W = 340;
const H = 100;
const PAD = { top: 8, right: 8, bottom: 24, left: 36 };

export default function YieldChart({ data, tier }: YieldChartProps) {
  if (data.length < 2) {
    return (
      <p style={{ fontSize: '0.78rem', color: '#94a3b8', textAlign: 'center', padding: '1rem 0' }}>
        Chart available after first harvest
      </p>
    );
  }

  const values = data.map((d) => parseFloat(d.cumulativeYield));
  const maxVal = Math.max(...values);
  const innerW = W - PAD.left - PAD.right;
  const innerH = H - PAD.top - PAD.bottom;

  const pts = data.map((d, i) => {
    const x = PAD.left + (i / (data.length - 1)) * innerW;
    const y = PAD.top + innerH - (parseFloat(d.cumulativeYield) / maxVal) * innerH;
    return `${x},${y}`;
  });
  const polyline = pts.join(' ');

  // filled area path
  const first = pts[0].split(',');
  const last = pts[pts.length - 1].split(',');
  const area = `M${first[0]},${parseFloat(first[1])} L${pts.join(' L')} L${last[0]},${PAD.top + innerH} L${PAD.left},${PAD.top + innerH} Z`;

  const yLabels = [0, maxVal / 2, maxVal].reverse();

  return (
    <svg
      width="100%"
      viewBox={`0 0 ${W} ${H}`}
      aria-label={`Cumulative yield chart for ${tier}`}
      style={{ display: 'block', overflow: 'visible' }}
    >
      <defs>
        <linearGradient id={`grad-${tier}`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#3b82f6" stopOpacity="0.25" />
          <stop offset="100%" stopColor="#3b82f6" stopOpacity="0" />
        </linearGradient>
      </defs>

      {/* Y-axis labels */}
      {yLabels.map((v, i) => {
        const y = PAD.top + (i / (yLabels.length - 1)) * innerH;
        return (
          <text key={i} x={PAD.left - 4} y={y + 4} textAnchor="end" fontSize="9" fill="#64748b">
            {v.toFixed(1)}
          </text>
        );
      })}

      {/* X-axis labels: first and last date */}
      <text x={PAD.left} y={H - 4} fontSize="9" fill="#64748b">{data[0].date.slice(5)}</text>
      <text x={W - PAD.right} y={H - 4} fontSize="9" fill="#64748b" textAnchor="end">
        {data[data.length - 1].date.slice(5)}
      </text>

      {/* Area fill */}
      <path d={area} fill={`url(#grad-${tier})`} />

      {/* Line */}
      <polyline points={polyline} fill="none" stroke="#3b82f6" strokeWidth="1.5" strokeLinejoin="round" />

      {/* Dots */}
      {pts.map((pt, i) => {
        const [x, y] = pt.split(',');
        return <circle key={i} cx={x} cy={y} r="2.5" fill="#3b82f6" />;
      })}
    </svg>
  );
}
