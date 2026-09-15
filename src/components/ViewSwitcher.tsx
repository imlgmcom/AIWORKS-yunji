// 视图切换器 + 主题切换（浮动在内容区右下角）
import type { ReactElement } from 'react';
import { useLocation } from 'react-router-dom';
import { Button, Tooltip } from '@fluentui/react-components';
import {
  AppsListRegular,
  GridRegular,
  GridDotsRegular,
  TextBulletListRegular,
  WeatherSunnyRegular,
  WeatherMoonRegular,
} from '@fluentui/react-icons';
import { useUIStore, type ViewMode } from '../stores/uiStore';
import { useSharedStyles } from '../styles/shared';

const VIEW_MODES: { mode: ViewMode; icon: ReactElement; label: string }[] = [
  { mode: 'masonry', icon: <GridDotsRegular />, label: '瀑布流' },
  { mode: 'card-v', icon: <GridRegular />, label: '竖向卡片' },
  { mode: 'card-h', icon: <AppsListRegular />, label: '横向卡片' },
  { mode: 'table', icon: <TextBulletListRegular />, label: '表格列表' },
];

export function ViewSwitcher() {
  const s = useSharedStyles();
  const { viewMode, setViewMode, theme, toggleTheme } = useUIStore();
  const location = useLocation();
  // 仅在支持多视图的文章列表页（全部列表、回收站）显示视图切换按钮
  const showViewModes = location.pathname === '/' || location.pathname === '/trash';

  return (
    <div className={s.viewSwitcher}>
      {showViewModes &&
        VIEW_MODES.map(({ mode, icon, label }) => (
          <Tooltip key={mode} content={label} relationship="label">
            <Button
              icon={icon}
              size="small"
              appearance="subtle"
              className={`${s.viewSwitcherBtn} ${viewMode === mode ? s.viewSwitcherActive : ''}`}
              onClick={() => setViewMode(mode)}
            />
          </Tooltip>
        ))}
      {showViewModes && <span className={s.viewSwitcherDivider} />}
      <Tooltip content={theme === 'light' ? '切换暗色' : '切换亮色'} relationship="label">
        <Button
          icon={theme === 'light' ? <WeatherSunnyRegular /> : <WeatherMoonRegular />}
          size="small"
          appearance="subtle"
          className={s.viewSwitcherBtn}
          onClick={toggleTheme}
        />
      </Tooltip>
    </div>
  );
}
