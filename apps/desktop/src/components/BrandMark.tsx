/** App brand mark — lime accent, dimensional monogram. */
export function BrandMark({ className = 'brand-mark' }: { className?: string }) {
  return (
    <span className={className} aria-hidden="true">
      <svg viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg" focusable="false">
        <defs>
          <linearGradient id="bm-plate" x1="18%" y1="8%" x2="86%" y2="94%">
            <stop offset="0%" stopColor="#f4ffb0" />
            <stop offset="38%" stopColor="#d6f078" />
            <stop offset="72%" stopColor="#9ed63a" />
            <stop offset="100%" stopColor="#5f9a28" />
          </linearGradient>
          <linearGradient id="bm-rim" x1="0%" y1="100%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#7fd4cc" stopOpacity="0.55" />
            <stop offset="45%" stopColor="#7fd4cc" stopOpacity="0" />
            <stop offset="100%" stopColor="#fff6a8" stopOpacity="0.35" />
          </linearGradient>
          <linearGradient id="bm-sheen" x1="50%" y1="0%" x2="50%" y2="55%">
            <stop offset="0%" stopColor="#ffffff" stopOpacity="0.72" />
            <stop offset="55%" stopColor="#ffffff" stopOpacity="0.08" />
            <stop offset="100%" stopColor="#ffffff" stopOpacity="0" />
          </linearGradient>
          <linearGradient id="bm-glyph" x1="30%" y1="20%" x2="70%" y2="90%">
            <stop offset="0%" stopColor="#1a220f" />
            <stop offset="100%" stopColor="#0c1208" />
          </linearGradient>
          <linearGradient id="bm-bevel" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stopColor="#ffffff" stopOpacity="0.42" />
            <stop offset="18%" stopColor="#ffffff" stopOpacity="0.12" />
            <stop offset="82%" stopColor="#000000" stopOpacity="0" />
            <stop offset="100%" stopColor="#000000" stopOpacity="0.22" />
          </linearGradient>
          <clipPath id="bm-clip">
            <rect x="14" y="14" width="100" height="100" rx="28" ry="28" />
          </clipPath>
        </defs>

        <rect x="14" y="14" width="100" height="100" rx="28" ry="28" fill="url(#bm-plate)" />
        <rect x="14" y="14" width="100" height="100" rx="28" ry="28" fill="url(#bm-rim)" />
        <rect x="14" y="14" width="100" height="100" rx="28" ry="28" fill="url(#bm-bevel)" />

        <g clipPath="url(#bm-clip)">
          <path d="M14 42 C30 22, 98 18, 114 36 L114 14 L14 14 Z" fill="url(#bm-sheen)" />
          <circle cx="40" cy="38" r="28" fill="#ffffff" opacity="0.12" />
        </g>

        <rect
          x="15.5"
          y="15.5"
          width="97"
          height="97"
          rx="26.5"
          ry="26.5"
          fill="none"
          stroke="#f7ffc8"
          strokeOpacity="0.55"
          strokeWidth="1.5"
        />

        <path
          d="M44 36
             H72
             C84.5 36 92 43.2 92 54.2
             C92 62.8 87.2 68.6 79.2 70.8
             L94 92
             H81.2
             L67.4 72.2
             H56.5
             V92
             H44
             Z
             M56.5 47.2
             V61.4
             H70.6
             C76.4 61.4 79.8 58.4 79.8 53.8
             C79.8 49.2 76.4 47.2 70.6 47.2
             Z"
          fill="url(#bm-glyph)"
          fillRule="evenodd"
        />
        <path
          d="M45.5 37.5
             H71.2
             C82.6 37.5 90 44.2 90 54.2
             C90 61.8 86 67 79 69.2"
          fill="none"
          stroke="#f0ffb0"
          strokeOpacity="0.28"
          strokeWidth="1.4"
          strokeLinecap="round"
        />

        <circle cx="93" cy="34" r="5.2" fill="#8ec5c0" />
        <circle cx="93" cy="34" r="2.4" fill="#f7ffef" />
        <circle
          cx="93"
          cy="34"
          r="7.5"
          fill="none"
          stroke="#8ec5c0"
          strokeOpacity="0.45"
          strokeWidth="1.2"
        />
      </svg>
    </span>
  );
}
