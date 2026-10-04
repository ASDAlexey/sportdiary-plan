import type { ReactNode } from 'react';

type Block =
  | { type: 'heading'; level: number; text: string }
  | { type: 'paragraph'; text: string }
  | { type: 'list'; ordered: boolean; items: string[] }
  | { type: 'table'; header: string[]; rows: string[][] }
  | { type: 'quote'; text: string };

function splitRow(line: string): string[] {
  return line
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split('|')
    .map((c) => c.trim());
}

function isSeparator(line: string): boolean {
  return /^\|?[\s:-]*-[-\s:|]*\|?$/.test(line) && line.includes('-');
}

function parseBlocks(src: string): Block[] {
  const lines = src.replace(/\r\n/g, '\n').split('\n');
  const blocks: Block[] = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    const trimmed = line.trim();
    if (!trimmed) {
      i += 1;
      continue;
    }
    const h = /^(#{1,4})\s+(.*)$/.exec(trimmed);
    if (h) {
      blocks.push({ type: 'heading', level: h[1].length, text: h[2] });
      i += 1;
      continue;
    }
    if (trimmed.startsWith('|') && i + 1 < lines.length && isSeparator(lines[i + 1].trim())) {
      const header = splitRow(trimmed);
      i += 2;
      const rows: string[][] = [];
      while (i < lines.length && lines[i].trim().startsWith('|')) {
        rows.push(splitRow(lines[i].trim()));
        i += 1;
      }
      blocks.push({ type: 'table', header, rows });
      continue;
    }
    const ul = /^[-*]\s+(.*)$/.exec(trimmed);
    const ol = /^\d+[.)]\s+(.*)$/.exec(trimmed);
    if (ul || ol) {
      const ordered = !!ol;
      const items: string[] = [];
      while (i < lines.length) {
        const t = lines[i].trim();
        const m = ordered ? /^\d+[.)]\s+(.*)$/.exec(t) : /^[-*]\s+(.*)$/.exec(t);
        if (!m) break;
        items.push(m[1]);
        i += 1;
      }
      blocks.push({ type: 'list', ordered, items });
      continue;
    }
    if (trimmed.startsWith('>')) {
      const parts: string[] = [];
      while (i < lines.length && lines[i].trim().startsWith('>')) {
        parts.push(lines[i].trim().replace(/^>\s?/, ''));
        i += 1;
      }
      blocks.push({ type: 'quote', text: parts.join(' ') });
      continue;
    }
    const para: string[] = [];
    while (i < lines.length) {
      const t = lines[i].trim();
      if (!t || t.startsWith('#') || t.startsWith('|') || t.startsWith('>') || /^[-*]\s+/.test(t) || /^\d+[.)]\s+/.test(t)) break;
      para.push(t);
      i += 1;
    }
    blocks.push({ type: 'paragraph', text: para.join(' ') });
  }
  return blocks;
}

function renderInline(text: string, keyPrefix: string): ReactNode[] {
  const tokens = text.split(/(\*\*[^*]+\*\*|\*[^*]+\*|`[^`]+`|\[[^\]]+\]\([^)\s]+\))/g);
  return tokens.map((tok, i) => {
    if (!tok) return null;
    if (tok.startsWith('**') && tok.endsWith('**')) return <strong key={`${keyPrefix}-${i}`}>{tok.slice(2, -2)}</strong>;
    if (tok.startsWith('*') && tok.endsWith('*')) return <em key={`${keyPrefix}-${i}`}>{tok.slice(1, -1)}</em>;
    if (tok.startsWith('`') && tok.endsWith('`')) return <code key={`${keyPrefix}-${i}`}>{tok.slice(1, -1)}</code>;
    const link = /^\[([^\]]+)\]\(([^)\s]+)\)$/.exec(tok);
    if (link) {
      return (
        <a key={`${keyPrefix}-${i}`} href={link[2]}>
          {link[1]}
        </a>
      );
    }
    return <span key={`${keyPrefix}-${i}`}>{tok}</span>;
  });
}

export function Markdown({ source }: { source: string }) {
  const blocks = parseBlocks(source);
  return (
    <div className="md">
      {blocks.map((b, i) => {
        const key = `b-${i}`;
        if (b.type === 'heading') {
          const Tag = (`h${Math.min(b.level + 1, 4)}`) as 'h2' | 'h3' | 'h4' | 'h5';
          return <Tag key={key}>{renderInline(b.text, key)}</Tag>;
        }
        if (b.type === 'paragraph') return <p key={key}>{renderInline(b.text, key)}</p>;
        if (b.type === 'quote') return <blockquote key={key}>{renderInline(b.text, key)}</blockquote>;
        if (b.type === 'list') {
          const items = b.items.map((it, j) => <li key={`${key}-${j}`}>{renderInline(it, `${key}-${j}`)}</li>);
          return b.ordered ? <ol key={key}>{items}</ol> : <ul key={key}>{items}</ul>;
        }
        return (
          <div className="table-scroll" key={key}>
            <table className="md-table">
              <thead>
                <tr>
                  {b.header.map((c, j) => (
                    <th key={`${key}-h-${j}`}>{renderInline(c, `${key}-h-${j}`)}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {b.rows.map((row, r) => (
                  <tr key={`${key}-r-${r}`}>
                    {row.map((c, cIdx) => (
                      <td key={`${key}-r-${r}-${cIdx}`}>{renderInline(c, `${key}-r-${r}-${cIdx}`)}</td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        );
      })}
    </div>
  );
}
