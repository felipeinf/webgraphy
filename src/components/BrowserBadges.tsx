interface BrowserBadgesProps {
  browsers: string[];
}

const BROWSER_COLORS: Record<string, string> = {
  Safari: "#5ac8fa",
  Chrome: "#fbbc04",
  Opera: "#ff1b2d",
};

export function BrowserBadges({ browsers }: BrowserBadgesProps) {
  if (browsers.length === 0) return null;

  return (
    <div className="browser-badges">
      {browsers.map((browser) => (
        <span
          key={browser}
          className="browser-badge"
          style={{ borderColor: BROWSER_COLORS[browser] ?? "#888" }}
        >
          {browser}
        </span>
      ))}
    </div>
  );
}
