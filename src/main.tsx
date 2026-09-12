// 应用入口：FluentProvider + QueryClientProvider + RouterProvider
import React from 'react';
import ReactDOM from 'react-dom/client';
import { FluentProvider } from '@fluentui/react-components';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { RouterProvider } from 'react-router-dom';
import { yunjiLightTheme, yunjiDarkTheme } from './theme/fluentTheme';
import { router } from './App';
import { useUIStore } from './stores/uiStore';
import { useAuthStore } from './stores/authStore';
import { initAssetBaseDir } from './lib/tauri';
import './styles/global.css';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function AppContent() {
  const theme = useUIStore((s) => s.theme);
  const init = useAuthStore((s) => s.init);

  React.useEffect(() => {
    // 并行初始化认证状态和资源根目录
    Promise.all([init(), initAssetBaseDir()]).catch(() => {});
    // 禁用 Tauri/WebView 默认右键菜单；有操作权限的卡片由 Fluent UI openOnContext 接管
    const handler = (e: MouseEvent) => {
      e.preventDefault();
    };
    document.addEventListener('contextmenu', handler);
    return () => document.removeEventListener('contextmenu', handler);
  }, [init]);

  return (
    <FluentProvider theme={theme === 'dark' ? yunjiDarkTheme : yunjiLightTheme}>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
    </FluentProvider>
  );
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <AppContent />
  </React.StrictMode>,
);
