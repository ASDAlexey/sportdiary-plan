import type { Metadata } from 'next';
import Nav from '@/components/Nav';
import { loadWorkouts } from '@/lib/data';
import { fmtDayYear, todayISO } from '@/lib/dates';
import './globals.css';

export const metadata: Metadata = {
  title: {
    default: 'Беговой журнал',
    template: '%s · Беговой журнал',
  },
  description: 'Дашборд тренировочного процесса: план, объёмы, гонки и цель на 5 км.',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  const builtAt = fmtDayYear(todayISO());
  const workoutCount = loadWorkouts().length;

  return (
    <html lang="ru">
      <body>
        <Nav />
        <main className="container">{children}</main>
        <footer className="footer">
          <div className="container">
            Собрано {builtAt} · тренировок в данных: {workoutCount}
          </div>
        </footer>
      </body>
    </html>
  );
}
