'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';

const LINKS = [
  { href: '/', label: 'Дашборд' },
  { href: '/workouts', label: 'Тренировки' },
  { href: '/plan', label: 'План' },
  { href: '/goal', label: 'Цель' },
  { href: '/races', label: 'Гонки' },
];

export default function Nav() {
  const pathname = usePathname() ?? '/';
  const norm = pathname.replace(/\/+$/, '') || '/';
  return (
    <nav className="nav" aria-label="Основная навигация">
      <div className="nav-inner">
        <Link href="/" className="nav-brand">
          Беговой журнал
        </Link>
        <ul className="nav-links">
          {LINKS.map((l) => {
            const active = l.href === '/' ? norm === '/' : norm === l.href || norm.startsWith(`${l.href}/`);
            return (
              <li key={l.href}>
                <Link href={l.href} className={active ? 'nav-link active' : 'nav-link'} aria-current={active ? 'page' : undefined}>
                  {l.label}
                </Link>
              </li>
            );
          })}
        </ul>
      </div>
    </nav>
  );
}
