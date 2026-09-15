// 设置页：Tab 切换（常规 / 用户管理 / 权限管理 / 附件清理）
import { useEffect, useMemo, useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Avatar,
  Badge,
  Button,
  Card,
  Checkbox,
  Dialog,
  DialogSurface,
  DialogBody,
  DialogTitle,
  DialogContent,
  DialogActions,
  Dropdown,
  Option,
  Input,
  Label,
  MessageBar,
  MessageBarBody,
  MessageBarTitle,
  Radio,
  RadioGroup,
  Spinner,
  Switch,
  Tab,
  TabList,
  Textarea,
  makeStyles,
  tokens,
} from '@fluentui/react-components';
import {
  SettingsRegular,
  CloudRegular,
  PersonRegular,
  DeleteRegular,
  EditRegular,
  ShieldRegular,
  ScanObjectRegular,
  FolderOpenRegular,
  ImageRegular,
  DocumentRegular,
  DesktopRegular,
  SearchRegular,
} from '@fluentui/react-icons';
import { settingsApi, userApi, authApi, assetUrl, cleanupApi, torrentApi } from '../lib/tauri';
import type { OrphanFile } from '../lib/tauri';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { useAuthStore } from '../stores/authStore';
import { usePerms } from '../hooks/usePerms';
import { useSelection, BatchBar } from '../components/batch';
import { AvatarCropDialog } from '../components/AvatarCropDialog';
import { Pagination } from '../components/Pagination';
import { formatBytes } from '../lib/format';
import type { AdminUser } from '../types/models';

// 0-5 权限选项文案
export const LEVEL_OPTIONS = [0, 1, 2, 3, 4, 5];
export const levelText = (n: number) =>
  n === 0 ? '0 - 所有登录用户' : n === 5 ? '5 - 仅管理员' : `权限 ${n} 级及以上`;

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '16px',
    maxWidth: '760px',
  },
  card: {
    padding: '16px',
  },
  title: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontSize: '16px',
    fontWeight: 700,
    marginBottom: '12px',
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    marginBottom: '12px',
  },
  userRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '8px 4px',
    borderBottom: `1px solid ${tokens.colorNeutralStroke2}`,
    '&:last-child': {
      borderBottom: 'none',
    },
  },
  userCheck: {
    flexShrink: 0,
  },
  userInfo: {
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
    minWidth: 0,
  },
  userName: {
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
    fontWeight: 600,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  userMeta: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  },
  cleanupContainer: {
    display: 'flex',
    flexDirection: 'column',
    gap: '12px',
    maxWidth: '960px',
  },
  filterBar: {
    display: 'flex',
    gap: '6px',
    flexWrap: 'wrap',
  },
  orphanRow: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '8px 4px',
    borderBottom: `1px solid ${tokens.colorNeutralStroke2}`,
    '&:last-child': {
      borderBottom: 'none',
    },
  },
  orphanCheck: {
    flexShrink: 0,
  },
  orphanThumb: {
    width: '36px',
    height: '36px',
    objectFit: 'cover',
    borderRadius: '4px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    backgroundColor: tokens.colorNeutralBackground2,
    flexShrink: 0,
  },
  orphanIcon: {
    width: '36px',
    height: '36px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: '4px',
    backgroundColor: tokens.colorNeutralBackground2,
    color: tokens.colorNeutralForeground3,
    flexShrink: 0,
  },
});

type TabId = 'general' | 'users' | 'permissions' | 'cleanup';

export function AdminPage() {
  const { isAdmin, can, settingsLoaded } = usePerms();
  const canUser = can('user');

  const tabs: { id: TabId; label: string }[] = [];
  if (isAdmin) tabs.push({ id: 'general', label: '常规设置' });
  if (canUser) tabs.push({ id: 'users', label: '用户管理' });
  if (isAdmin) tabs.push({ id: 'permissions', label: '权限管理' });
  if (isAdmin) tabs.push({ id: 'cleanup', label: '附件清理' });

  const [tab, setTab] = useState<TabId | null>(tabs[0]?.id ?? null);
  useEffect(() => {
    if (tab && tabs.some((t) => t.id === tab)) return;
    setTab(tabs[0]?.id ?? null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isAdmin, canUser]);

  if (!settingsLoaded && !isAdmin) {
    return <Spinner label="加载中..." />;
  }

  if (tabs.length === 0) {
    return (
      <MessageBar intent="warning">
        <MessageBarBody>
          <MessageBarTitle>无权访问</MessageBarTitle>
          当前账户没有任何设置项的访问权限。
        </MessageBarBody>
      </MessageBar>
    );
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
      <h2 style={{ margin: 0 }}>应用设置</h2>
      <TabList selectedValue={tab} onTabSelect={(_, d) => setTab(d.value as TabId)}>
        {tabs.map((t) => (
          <Tab key={t.id} value={t.id}>
            {t.label}
          </Tab>
        ))}
      </TabList>

      {tab === 'general' && isAdmin && <GeneralTab />}
      {tab === 'users' && canUser && <UsersTab />}
      {tab === 'permissions' && isAdmin && <PermissionsTab />}
      {tab === 'cleanup' && isAdmin && <CleanupTab />}
    </div>
  );
}

// --- 常规：站点 + 存储 ---

function GeneralTab() {
  const styles = useStyles();
  const queryClient = useQueryClient();
  const [form, setForm] = useState<Record<string, string>>({});
  const [logoBusy, setLogoBusy] = useState(false);
  const [logoError, setLogoError] = useState('');

  const { data: settings } = useQuery({
    queryKey: ['settings'],
    queryFn: settingsApi.get,
  });

  useEffect(() => {
    if (settings) setForm(settings);
  }, [settings]);

  const updateMut = useMutation({
    mutationFn: (s: Record<string, string>) => settingsApi.update(s),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
      queryClient.invalidateQueries({ queryKey: ['public-settings'] });
    },
  });

  const set = (k: string, v: string) => setForm((f) => ({ ...f, [k]: v }));

  const refreshSettings = () => {
    queryClient.invalidateQueries({ queryKey: ['settings'] });
    queryClient.invalidateQueries({ queryKey: ['public-settings'] });
  };

  const pickLogo = async (kind: 'image' | 'icon') => {
    setLogoError('');
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif', 'svg', 'ico'] }],
    });
    if (typeof selected !== 'string') return;
    setLogoBusy(true);
    try {
      const savedPath = await settingsApi.uploadLogo(selected, kind);
      set(kind === 'image' ? 'logo_image_path' : 'logo_icon_path', savedPath);
      refreshSettings();
    } catch (e: any) {
      setLogoError(e.message || 'LOGO 上传失败');
    } finally {
      setLogoBusy(false);
    }
  };

  const removeLogo = async (kind: 'image' | 'icon') => {
    setLogoError('');
    setLogoBusy(true);
    try {
      await settingsApi.clearLogo(kind);
      set(kind === 'image' ? 'logo_image_path' : 'logo_icon_path', '');
      refreshSettings();
    } catch (e: any) {
      setLogoError(e.message || 'LOGO 清除失败');
    } finally {
      setLogoBusy(false);
    }
  };

  const logoMode = form.logo_mode ?? 'text';
  const logoPath = form.logo_image_path ?? '';
  const iconPath = form.logo_icon_path ?? '';

  return (
    <div className={styles.container}>
      <Card className={styles.card}>
        <div className={styles.title}>
          <SettingsRegular /> 站点信息
        </div>
        <div className={styles.field}>
          <Label htmlFor="site_name">站点名称</Label>
          <Input
            id="site_name"
            value={form.site_name ?? ''}
            onChange={(_, d) => set('site_name', d.value)}
          />
        </div>
        <div className={styles.field}>
          <Label id="logo_mode_label">LOGO 样式</Label>
          <RadioGroup
            aria-labelledby="logo_mode_label"
            value={logoMode}
            onChange={(_, d) => set('logo_mode', d.value)}
          >
            <Radio value="text" label="文字 LOGO（仅显示站点名称）" />
            <Radio value="icon_text" label="图标 + 文字" />
            <Radio value="image" label="图片 LOGO" />
          </RadioGroup>
        </div>
        {(logoMode === 'image' || logoMode === 'icon_text') && (
          <div className={styles.field}>
            <Label>{logoMode === 'image' ? 'LOGO 图片' : '图标图片'}</Label>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <div
                style={
                  logoMode === 'image'
                    ? {
                        width: '120px',
                        height: '48px',
                        border: `1px solid ${tokens.colorNeutralStroke2}`,
                        borderRadius: '8px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        backgroundColor: tokens.colorNeutralBackground2,
                        overflow: 'hidden',
                      }
                    : {
                        width: '48px',
                        height: '48px',
                        border: `1px solid ${tokens.colorNeutralStroke2}`,
                        borderRadius: '8px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        backgroundColor: tokens.colorNeutralBackground2,
                        overflow: 'hidden',
                      }
                }
              >
                {logoMode === 'image' ? (
                  logoPath ? (
                    <img
                      src={assetUrl(logoPath)}
                      alt="LOGO"
                      style={{ maxWidth: '90%', maxHeight: '90%', objectFit: 'contain' }}
                    />
                  ) : (
                    <span style={{ fontSize: '12px', color: tokens.colorNeutralForeground3 }}>
                      未设置
                    </span>
                  )
                ) : iconPath ? (
                  <img
                    src={assetUrl(iconPath)}
                    alt="图标"
                    style={{ maxWidth: '90%', maxHeight: '90%', objectFit: 'contain' }}
                  />
                ) : (
                  <span style={{ fontSize: '11px', color: tokens.colorNeutralForeground3 }}>
                    默认图标
                  </span>
                )}
              </div>
              <Button
                appearance="outline"
                icon={<ImageRegular />}
                onClick={() => pickLogo(logoMode === 'image' ? 'image' : 'icon')}
                disabled={logoBusy}
              >
                {logoMode === 'image'
                  ? logoPath
                    ? '更换图片'
                    : '上传图片'
                  : iconPath
                    ? '更换图标'
                    : '上传图标'}
              </Button>
              {((logoMode === 'image' && logoPath) || (logoMode === 'icon_text' && iconPath)) && (
                <Button
                  appearance="subtle"
                  icon={<DeleteRegular />}
                  onClick={() => removeLogo(logoMode === 'image' ? 'image' : 'icon')}
                  disabled={logoBusy}
                >
                  移除
                </Button>
              )}
              {logoBusy && <Spinner size="tiny" />}
            </div>
            <div style={{ fontSize: '12px', color: tokens.colorNeutralForeground3, marginTop: '4px' }}>
              {logoMode === 'image'
                ? '图片将铺满左上角 LOGO 区域（建议宽幅图片）'
                : '建议使用正方形图片，显示为站点名称前的图标；不上传则使用默认图标'}
            </div>
            {logoError && (
              <div style={{ color: tokens.colorPaletteRedForeground1, fontSize: '12px', marginTop: '4px' }}>
                {logoError}
              </div>
            )}
          </div>
        )}
        <Button
          appearance="primary"
          onClick={() =>
            updateMut.mutate({
              site_name: form.site_name ?? '',
              logo_mode: logoMode,
            })
          }
          disabled={updateMut.isPending}
        >
          保存
        </Button>
      </Card>

      <Card className={styles.card}>
        <div className={styles.title}>
          <DesktopRegular /> 窗口行为
        </div>
        <div className={styles.field}>
          <Switch
            label="关闭按钮最小化到托盘（点击窗口 × 时隐藏到系统托盘，通过托盘菜单退出程序）"
            checked={form.close_to_tray === '1'}
            onChange={(_, d) => updateMut.mutate({ close_to_tray: d.checked ? '1' : '0' })}
          />
        </div>
      </Card>

      <Card className={styles.card}>
        <div className={styles.title}>
          <CloudRegular /> 存储后端
        </div>
        <div className={styles.field}>
          <Label htmlFor="local_upload_dir">本地存储目录（相对或绝对路径）</Label>
          <Input
            id="local_upload_dir"
            value={form.local_upload_dir ?? ''}
            onChange={(_, d) => set('local_upload_dir', d.value)}
          />
        </div>
        <div className={styles.field}>
          <Label htmlFor="local_url_prefix">URL 前缀（用于本地资源访问）</Label>
          <Input
            id="local_url_prefix"
            value={form.local_url_prefix ?? ''}
            onChange={(_, d) => set('local_url_prefix', d.value)}
          />
        </div>
        <Button
          appearance="primary"
          onClick={() =>
            updateMut.mutate({
              local_upload_dir: form.local_upload_dir ?? 'uploads',
              local_url_prefix: form.local_url_prefix ?? 'yunji-files',
            })
          }
          disabled={updateMut.isPending}
        >
          保存存储设置
        </Button>
      </Card>
    </div>
  );
}

// --- 用户管理 ---

interface EditForm {
  username: string;
  nickname: string;
  email: string;
  bio: string;
  permission_level: number;
  new_password: string;
}

function UsersTab() {
  const styles = useStyles();
  const queryClient = useQueryClient();
  const selection = useSelection();
  const currentUserId = useAuthStore((s) => s.user?.id);
  const refreshMe = useAuthStore((s) => s.setUser);
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null);
  const [editTarget, setEditTarget] = useState<AdminUser | null>(null);
  const [cropTarget, setCropTarget] = useState<{ userId: number; file: string } | null>(null);
  const [editForm, setEditForm] = useState<EditForm>({
    username: '', nickname: '', email: '', bio: '', permission_level: 0, new_password: '',
  });
  const [page, setPage] = useState(1);
  const [keyword, setKeyword] = useState('');
  const PAGE_SIZE = 36;

  const { data, isLoading } = useQuery({
    queryKey: ['users', 'paged', page, keyword],
    queryFn: () => userApi.listPage(page, PAGE_SIZE, keyword || undefined),
  });
  const users = data?.items ?? [];
  const totalUsers = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(totalUsers / PAGE_SIZE));

  // 删除后当前页空了自动回退；翻页/搜索清空选择
  useEffect(() => {
    if (!isLoading && page > 1 && users.length === 0) setPage(page - 1);
  }, [isLoading, users.length, page]);
  useEffect(() => {
    selection.clear();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page, keyword]);

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['users'] });

  const deleteMut = useMutation({
    mutationFn: (ids: number[]) => userApi.deleteMany(ids),
    onSuccess: () => {
      invalidate();
      setMsg({ ok: true, text: '用户已删除' });
    },
    onError: (e: Error) => setMsg({ ok: false, text: e.message || '删除失败' }),
  });

  const updateMut = useMutation({
    mutationFn: userApi.update,
    onSuccess: () => {
      invalidate();
      setEditTarget(null);
      setMsg({ ok: true, text: '用户信息已保存' });
    },
    onError: (e: Error) => setMsg({ ok: false, text: e.message || '保存失败' }),
  });

  const userIds = users.map((u) => u.id);

  const batchDelete = () => {
    const ids = Array.from(selection.selected);
    if (ids.length === 0) return;
    if (!confirm(`确认删除选中的 ${ids.length} 个用户？此操作不可恢复！`)) return;
    deleteMut.mutate(ids, { onSettled: () => selection.clear() });
  };

  const openEdit = (u: AdminUser) => {
    setMsg(null);
    setEditTarget(u);
    setEditForm({
      username: u.username,
      nickname: u.nickname,
      email: u.email,
      bio: u.bio,
      permission_level: u.is_admin ? 5 : u.permission_level,
      new_password: '',
    });
  };

  const saveEdit = () => {
    if (!editTarget) return;
    const editingSelf = editTarget.id === currentUserId;
    updateMut.mutate({
      user_id: editTarget.id,
      username: editForm.username.trim(),
      nickname: editForm.nickname.trim(),
      email: editForm.email.trim(),
      bio: editForm.bio,
      permission_level: editForm.permission_level,
      new_password: editForm.new_password.trim() || null,
    }, {
      onSuccess: async () => {
        // 编辑的是自己：刷新登录态，导航栏昵称等立即生效
        if (editingSelf) {
          const fresh = await authApi.me();
          if (fresh) refreshMe(fresh);
        }
      },
    });
  };

  const handlePickAvatar = async (userId: number) => {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif'] }],
    });
    const filePath = typeof selected === 'string' ? selected : null;
    if (filePath) setCropTarget({ userId, file: filePath });
  };

  return (
    <Card className={styles.card}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '12px', flexWrap: 'wrap' }}>
        <div className={styles.title}>
          <PersonRegular /> 用户管理
        </div>
        <Input
          style={{ width: '220px' }}
          placeholder="搜索用户名 / 昵称 / 邮箱"
          value={keyword}
          onChange={(_, d) => {
            setKeyword(d.value);
            setPage(1);
          }}
          contentBefore={<SearchRegular />}
        />
      </div>

      {msg && (
        <MessageBar intent={msg.ok ? 'success' : 'error'} style={{ marginBottom: '12px' }}>
          <MessageBarBody>
            <MessageBarTitle>{msg.ok ? '成功' : '失败'}</MessageBarTitle>
            {msg.text}
          </MessageBarBody>
        </MessageBar>
      )}

      {isLoading ? (
        <div style={{ color: tokens.colorNeutralForeground3, fontSize: '13px' }}>加载中...</div>
      ) : users.length === 0 ? (
        <div style={{ color: tokens.colorNeutralForeground3, fontSize: '13px', padding: '12px 0' }}>
          {keyword ? '没有匹配的用户' : '暂无用户'}
        </div>
      ) : (
        <div>
          {users.map((u) => (
            <div key={u.id} className={styles.userRow}>
              <Checkbox
                className={styles.userCheck}
                checked={selection.selected.has(u.id)}
                onChange={(_, d) => selection.toggle(u.id, d.checked === true)}
              />
              <Avatar
                size={36}
                name={u.nickname || u.username}
                image={u.avatar_path ? { src: assetUrl(u.avatar_path) } : undefined}
              />
              <div className={styles.userInfo}>
                <div className={styles.userName}>
                  {u.nickname || u.username}
                  <span style={{ fontWeight: 400, color: tokens.colorNeutralForeground3 }}>
                    (@{u.username})
                  </span>
                  {u.is_admin && <Badge size="small" color="brand">管理员</Badge>}
                  {!u.is_admin && <Badge size="small" appearance="outline">权限 {u.permission_level}</Badge>}
                  {u.id === currentUserId && <Badge size="small">当前用户</Badge>}
                </div>
                <div className={styles.userMeta}>
                  {u.email || '未设置邮箱'} · 注册于 {u.created_at.slice(0, 10)}
                </div>
              </div>
              <Button
                appearance="outline"
                size="small"
                icon={<EditRegular />}
                onClick={() => openEdit(u)}
              >
                编辑
              </Button>
            </div>
          ))}
          <Pagination page={page} totalPages={totalPages} total={totalUsers} onChange={setPage} />
        </div>
      )}

      {/* 批量操作栏 */}
      <BatchBar
        selectedCount={selection.size}
        allChecked={userIds.length > 0 && selection.size === userIds.length}
        total={userIds.length}
        onSelectAll={(c) => selection.selectAll(userIds, c)}
        onInvert={() => selection.invert(userIds)}
        onClear={selection.clear}
        busy={deleteMut.isPending}
      >
        <Button
          appearance="primary"
          size="small"
          icon={<DeleteRegular />}
          onClick={batchDelete}
          disabled={deleteMut.isPending}
        >
          批量删除
        </Button>
      </BatchBar>

      {/* 编辑用户对话框 */}
      <Dialog open={editTarget != null} onOpenChange={(_, d) => { if (!d.open) setEditTarget(null); }}>
        <DialogSurface>
          <DialogBody>
            <DialogTitle>
              编辑用户（{editTarget?.nickname || editTarget?.username}）
            </DialogTitle>
            <DialogContent>
              <div className={styles.field}>
                <Label>头像</Label>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <Avatar
                    size={56}
                    name={editForm.nickname || editForm.username}
                    image={editTarget?.avatar_path ? { src: assetUrl(editTarget.avatar_path) } : undefined}
                  />
                  <Button
                    appearance="outline"
                    size="small"
                    icon={<ImageRegular />}
                    onClick={() => editTarget && handlePickAvatar(editTarget.id)}
                  >
                    更换头像
                  </Button>
                </div>
              </div>
              <div className={styles.field}>
                <Label htmlFor="eu_username">用户名</Label>
                <Input
                  id="eu_username"
                  value={editForm.username}
                  onChange={(_, d) => setEditForm((f) => ({ ...f, username: d.value }))}
                />
              </div>
              <div className={styles.field}>
                <Label htmlFor="eu_nickname">昵称</Label>
                <Input
                  id="eu_nickname"
                  value={editForm.nickname}
                  onChange={(_, d) => setEditForm((f) => ({ ...f, nickname: d.value }))}
                />
              </div>
              <div className={styles.field}>
                <Label htmlFor="eu_email">邮箱</Label>
                <Input
                  id="eu_email"
                  value={editForm.email}
                  onChange={(_, d) => setEditForm((f) => ({ ...f, email: d.value }))}
                />
              </div>
              <div className={styles.field}>
                <Label htmlFor="eu_bio">简介</Label>
                <Textarea
                  id="eu_bio"
                  value={editForm.bio}
                  onChange={(_, d) => setEditForm((f) => ({ ...f, bio: d.value }))}
                />
              </div>
              <div className={styles.field}>
                <Label htmlFor="eu_level">权限值</Label>
                <Dropdown
                  id="eu_level"
                  value={editTarget?.is_admin ? '5 - 仅管理员（管理员固定 5）' : levelText(editForm.permission_level)}
                  selectedOptions={[String(editForm.permission_level)]}
                  disabled={!!editTarget?.is_admin}
                  onOptionSelect={(_, d) =>
                    d.optionValue && setEditForm((f) => ({ ...f, permission_level: Number(d.optionValue) }))
                  }
                >
                  {LEVEL_OPTIONS.map((n) => (
                    <Option key={n} value={String(n)} text={levelText(n)}>
                      {levelText(n)}
                    </Option>
                  ))}
                </Dropdown>
              </div>
              <div className={styles.field}>
                <Label htmlFor="eu_pwd">新密码（留空则不修改）</Label>
                <Input
                  id="eu_pwd"
                  type="password"
                  placeholder="至少 6 位"
                  value={editForm.new_password}
                  onChange={(_, d) => setEditForm((f) => ({ ...f, new_password: d.value }))}
                />
              </div>
            </DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={() => setEditTarget(null)}>
                取消
              </Button>
              <Button
                appearance="primary"
                disabled={
                  updateMut.isPending ||
                  !editForm.username.trim() ||
                  !editForm.nickname.trim() ||
                  (editForm.new_password.length > 0 && editForm.new_password.length < 6)
                }
                onClick={saveEdit}
              >
                保存
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>

      {/* 裁剪并设置目标用户头像 */}
      {cropTarget && (
        <AvatarCropDialog
          filePath={cropTarget.file}
          targetUserId={cropTarget.userId}
          onClose={() => setCropTarget(null)}
          onUploaded={async (path) => {
            invalidate();
            setEditTarget((t) => (t && t.id === cropTarget.userId ? { ...t, avatar_path: path } : t));
            setMsg({ ok: true, text: '头像已更新' });
            // 改的是自己：刷新登录态，导航栏头像立即生效
            if (cropTarget.userId === currentUserId) {
              const fresh = await authApi.me();
              if (fresh) refreshMe(fresh);
            }
          }}
        />
      )}
    </Card>
  );
}

// --- 权限管理 ---

const PERM_FIELDS: { key: string; label: string; hint?: string }[] = [
  { key: 'perm_article_own', label: '文章（自己）', hint: '新建、编辑、删除自己的文章' },
  { key: 'perm_article', label: '文章（管理他人）', hint: '编辑、删除、批量操作他人的文章' },
  { key: 'perm_collection_own', label: '合集（自己）', hint: '新建、编辑、删除自己的合集' },
  { key: 'perm_collection', label: '合集（管理他人）', hint: '管理他人创建的合集' },
  { key: 'perm_tag', label: '标签管理' },
  { key: 'perm_trash', label: '回收站管理' },
  { key: 'perm_user', label: '用户管理' },
];

function PermissionsTab() {
  const styles = useStyles();
  const queryClient = useQueryClient();
  const [form, setForm] = useState<Record<string, number>>({});
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null);

  const { data: settings } = useQuery({
    queryKey: ['settings'],
    queryFn: settingsApi.get,
  });

  useEffect(() => {
    if (settings) {
      const next: Record<string, number> = {};
      for (const f of PERM_FIELDS) {
        next[f.key] = Number(settings[f.key] ?? 5);
      }
      setForm(next);
    }
  }, [settings]);

  const saveMut = useMutation({
    mutationFn: (items: Record<string, string>) => settingsApi.update(items),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] });
      queryClient.invalidateQueries({ queryKey: ['public-settings'] });
      setMsg({ ok: true, text: '权限设置已保存' });
    },
    onError: (e: Error) => setMsg({ ok: false, text: e.message || '保存失败' }),
  });

  const save = () => {
    const items: Record<string, string> = {};
    for (const f of PERM_FIELDS) items[f.key] = String(form[f.key] ?? 5);
    saveMut.mutate(items);
  };

  return (
    <Card className={styles.card}>
      <div className={styles.title}>
        <ShieldRegular /> 权限管理
      </div>
      <p style={{ marginTop: 0, fontSize: '13px', color: tokens.colorNeutralForeground2 }}>
        设置各管理功能所需的最低权限值（0-5）。用户的权限值达到或超过该值即可使用；管理员始终拥有全部权限；默认 5（仅管理员）。
      </p>
      {PERM_FIELDS.map((f) => (
        <div key={f.key} className={styles.field}>
          <Label>{f.label}</Label>
          {f.hint && (
            <span style={{ fontSize: '12px', color: tokens.colorNeutralForeground3, display: 'block', marginBottom: '4px' }}>
              {f.hint}
            </span>
          )}
          <Dropdown
            value={levelText(form[f.key] ?? 5)}
            selectedOptions={[String(form[f.key] ?? 5)]}
            onOptionSelect={(_, d) =>
              d.optionValue && setForm((prev) => ({ ...prev, [f.key]: Number(d.optionValue) }))
            }
          >
            {LEVEL_OPTIONS.map((n) => (
              <Option key={n} value={String(n)} text={levelText(n)}>
                {levelText(n)}
              </Option>
            ))}
          </Dropdown>
        </div>
      ))}
      {msg && (
        <MessageBar intent={msg.ok ? 'success' : 'error'} style={{ marginBottom: '12px' }}>
          <MessageBarBody>
            <MessageBarTitle>{msg.ok ? '成功' : '失败'}</MessageBarTitle>
            {msg.text}
          </MessageBarBody>
        </MessageBar>
      )}
      <Button appearance="primary" onClick={save} disabled={saveMut.isPending}>
        保存权限设置
      </Button>
    </Card>
  );
}

// --- 附件清理 ---

const CATEGORY_LABELS: Record<OrphanFile['category'], string> = {
  thumbnail: '缩略图',
  gallery: '图集图片',
  content: '正文文件',
  file: '普通附件',
  torrent: '上传种子',
  magnet: '磁力种子',
  avatar: '头像',
  logo: '站点 LOGO',
  other: '其他',
};

function CleanupTab() {
  const styles = useStyles();
  const selection = useSelection();
  const [msg, setMsg] = useState<{ ok: boolean; text: string } | null>(null);
  const [cat, setCat] = useState<'all' | OrphanFile['category']>('all');

  // 手动触发扫描（enabled: false，进页面不扫）
  const { data, isFetching, refetch } = useQuery({
    queryKey: ['orphan-files'],
    queryFn: cleanupApi.scan,
    enabled: false,
  });
  const files = data?.files ?? [];

  const counts = useMemo(() => {
    const m = new Map<string, number>();
    for (const f of files) m.set(f.category, (m.get(f.category) ?? 0) + 1);
    return m;
  }, [files]);

  const filtered = useMemo(
    () => (cat === 'all' ? files : files.filter((f) => f.category === cat)),
    [files, cat],
  );

  // 切换筛选时清空选择（选择基于当前列表的序号）
  const switchCat = (c: 'all' | OrphanFile['category']) => {
    setCat(c);
    selection.clear();
  };

  const deleteMut = useMutation({
    mutationFn: (paths: string[]) => cleanupApi.deleteFiles(paths),
    onSuccess: (res) => {
      selection.clear();
      // enabled:false 的查询 invalidate 不会自动重取，手动刷新
      refetch();
      setMsg({
        ok: res.failed.length === 0,
        text:
          res.failed.length === 0
            ? `已删除 ${res.deleted.length} 个文件`
            : `已删除 ${res.deleted.length} 个，${res.failed.length} 个失败（${res.failed
                .slice(0, 3)
                .map((f) => f.error)
                .join('、')}）`,
      });
    },
    onError: (e: Error) => setMsg({ ok: false, text: e.message || '删除失败' }),
  });

  const batchDelete = () => {
    const paths = filtered
      .filter((_, i) => selection.selected.has(i))
      .map((f) => f.path);
    if (paths.length === 0) return;
    if (
      !confirm(
        `确认删除选中的 ${paths.length} 个文件？\n此操作不可恢复，若文章仍在引用这些文件，对应图片/附件将无法显示或打开。`,
      )
    ) {
      return;
    }
    deleteMut.mutate(paths);
  };

  const allChecked = filtered.length > 0 && filtered.every((_, i) => selection.selected.has(i));
  const busy = isFetching || deleteMut.isPending;

  return (
    <div className={styles.cleanupContainer}>
      <Card className={styles.card}>
        <div className={styles.title}>
          <ScanObjectRegular /> 附件清理
        </div>
        <MessageBar intent="info" style={{ marginBottom: '12px' }}>
          <MessageBarBody>
            扫描存储目录中数据库已无任何引用的文件：永久删除文章后的残留（缩略图、图集、种子等），
            以及上传到正文但后来不再引用的图片/附件。<b>回收站中的文章文件不会被列为垃圾</b>。
            勾选后删除，未勾选即保留。
          </MessageBarBody>
        </MessageBar>

        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <Button
            appearance="primary"
            icon={<ScanObjectRegular />}
            onClick={() => {
              setMsg(null);
              selection.clear();
              refetch();
            }}
            disabled={busy}
          >
            {data ? '重新扫描' : '开始扫描'}
          </Button>
          {isFetching && <Spinner size="tiny" label="扫描中（多线程遍历存储目录）..." />}
          {data && !isFetching && (
            <span style={{ fontSize: '13px', color: tokens.colorNeutralForeground3 }}>
              发现 {files.length} 个未引用文件，共 {formatBytes(data.total_size)}
            </span>
          )}
        </div>

        {msg && (
          <MessageBar intent={msg.ok ? 'success' : 'error'} style={{ marginTop: '12px' }}>
            <MessageBarBody>
              <MessageBarTitle>{msg.ok ? '完成' : '失败'}</MessageBarTitle>
              {msg.text}
            </MessageBarBody>
          </MessageBar>
        )}
      </Card>

      {data && !isFetching && files.length === 0 && (
        <Card className={styles.card}>
          <MessageBar intent="success">
            <MessageBarBody>未发现未引用文件，存储目录很干净。</MessageBarBody>
          </MessageBar>
        </Card>
      )}

      {filtered.length > 0 && (
        <Card className={styles.card}>
          <div className={styles.filterBar}>
            <Button
              size="small"
              appearance={cat === 'all' ? 'primary' : 'outline'}
              onClick={() => switchCat('all')}
            >
              全部（{files.length}）
            </Button>
            {Array.from(counts.entries())
              .sort((a, b) => b[1] - a[1])
              .map(([c, n]) => (
                <Button
                  key={c}
                  size="small"
                  appearance={cat === c ? 'primary' : 'outline'}
                  onClick={() => switchCat(c as OrphanFile['category'])}
                >
                  {CATEGORY_LABELS[c as OrphanFile['category']]}（{n}）
                </Button>
              ))}
          </div>

          <div style={{ marginTop: '8px' }}>
            {filtered.map((f, i) => (
              <div key={f.path} className={styles.orphanRow}>
                <Checkbox
                  className={styles.orphanCheck}
                  checked={selection.selected.has(i)}
                  onChange={(_, d) => selection.toggle(i, d.checked === true)}
                />
                {f.is_image ? (
                  <img className={styles.orphanThumb} src={assetUrl(f.path)} alt="" loading="lazy" />
                ) : (
                  <div className={styles.orphanIcon}>
                    {f.category === 'thumbnail' || f.category === 'gallery' ? (
                      <ImageRegular />
                    ) : (
                      <DocumentRegular />
                    )}
                  </div>
                )}
                <div className={styles.userInfo}>
                  <div className={styles.userName}>
                    {f.name}
                    <Badge size="small" appearance="outline">
                      {CATEGORY_LABELS[f.category]}
                    </Badge>
                    <span style={{ fontWeight: 400, fontSize: '12px' }}>{formatBytes(f.size)}</span>
                  </div>
                  <div className={styles.userMeta}>
                    {f.path} · {new Date(f.modified * 1000).toLocaleString()}
                  </div>
                </div>
                <Button
                  appearance="subtle"
                  size="small"
                  icon={<FolderOpenRegular />}
                  title="在资源管理器中定位"
                  onClick={() => torrentApi.revealFile(f.path).catch(() => undefined)}
                />
              </div>
            ))}
          </div>
        </Card>
      )}

      <BatchBar
        selectedCount={selection.size}
        allChecked={allChecked}
        total={filtered.length}
        onSelectAll={(c) => selection.selectAll(filtered.map((_, i) => i), c)}
        onInvert={() => selection.invert(filtered.map((_, i) => i))}
        onClear={selection.clear}
        busy={busy}
      >
        <Button
          appearance="primary"
          size="small"
          icon={<DeleteRegular />}
          onClick={batchDelete}
          disabled={deleteMut.isPending}
        >
          删除选中
        </Button>
      </BatchBar>
    </div>
  );
}
