// 认证守卫：未登录跳转登录页；feature 模式下还需对应功能权限
import { Navigate, useLocation } from 'react-router-dom';
import { useAuthStore } from '../stores/authStore';
import { MessageBar, MessageBarBody, MessageBarTitle, Spinner } from '@fluentui/react-components';
import { usePerms, type Feature } from '../hooks/usePerms';

export function RequireAuth({ children }: { children: React.ReactNode }) {
  const { user, loading } = useAuthStore();
  const location = useLocation();

  if (loading) {
    return <Spinner label="加载中..." />;
  }

  if (!user) {
    const next = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?next=${next}`} replace />;
  }

  return <>{children}</>;
}

const FEATURE_LABEL: Record<Feature, string> = {
  article_own: '文章（自己）',
  article: '文章（管理他人）',
  collection_own: '合集（自己）',
  collection: '合集（管理他人）',
  tag: '标签管理',
  trash: '回收站管理',
  user: '用户管理',
};

export function RequirePerm({ feature, children }: { feature: Feature; children: React.ReactNode }) {
  const { user, loading } = useAuthStore();
  const { can, settingsLoaded } = usePerms();
  const location = useLocation();

  if (loading) {
    return <Spinner label="加载中..." />;
  }
  if (!user) {
    const next = encodeURIComponent(location.pathname + location.search);
    return <Navigate to={`/login?next=${next}`} replace />;
  }
  // 管理员无需读取阈值；普通用户等待公开设置加载完成再判定
  if (!settingsLoaded && !user.is_admin) {
    return <Spinner label="加载中..." />;
  }
  if (!can(feature)) {
    return (
      <MessageBar intent="warning">
        <MessageBarBody>
          <MessageBarTitle>无权访问</MessageBarTitle>
          需要「{FEATURE_LABEL[feature]}」权限。
        </MessageBarBody>
      </MessageBar>
    );
  }
  return <>{children}</>;
}
