// 应用外壳：左侧侧边栏 + Outlet
import { Outlet } from 'react-router-dom';
import { Navbar } from './Navbar';
import { ViewSwitcher } from '../ViewSwitcher';
import { makeStyles, tokens } from '@fluentui/react-components';

const useStyles = makeStyles({
  shell: {
    minHeight: '100vh',
    backgroundColor: tokens.colorNeutralBackground2,
  },
  main: {
    padding: '20px 28px 20px 248px',
    overflowY: 'auto',
    minHeight: '100vh',
  },
});

export function AppShell() {
  const styles = useStyles();
  return (
    <div className={styles.shell}>
      <Navbar />
      <main className={styles.main}>
        <Outlet />
      </main>
      <ViewSwitcher />
    </div>
  );
}
