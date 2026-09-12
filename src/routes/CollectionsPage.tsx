// 合集管理页（自定义分类）
import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Input,
  Button,
  Label,
  Textarea,
  Dropdown,
  Option,
  Badge,
  Spinner,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogBody,
  DialogTitle,
  DialogContent,
  DialogActions,
  DialogOpenChangeData,
  tokens,
} from '@fluentui/react-components';
import {
  AddRegular,
  RenameRegular,
  DeleteRegular,
  ArrowUploadRegular,
  SearchRegular,
} from '@fluentui/react-icons';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { collectionApi, imageApi, resourceApi, assetUrl } from '../lib/tauri';
import type { Collection, CreateCollection, Resource } from '../types/models';
import { useSelection, CardCheckbox, BatchBar, runBatchAction } from '../components/batch';
import { CardContextMenu } from '../components/CardContextMenu';
import { useSharedStyles } from '../styles/shared';
import { usePerms } from '../hooks/usePerms';

interface EditState {
  id: number | null;
  title: string;
  description: string;
  visibility: string;
  accessPassword: string;
  thumbnailPath: string;
}

const emptyEdit: EditState = {
  id: null,
  title: '',
  description: '',
  visibility: 'private',
  accessPassword: '',
  thumbnailPath: '',
};

export function CollectionsPage() {
  const s = useSharedStyles();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const canCollection = usePerms().can('collection');
  const selection = useSelection();
  const [editOpen, setEditOpen] = useState(false);
  const [editState, setEditState] = useState<EditState>(emptyEdit);
  const [error, setError] = useState('');
  const [editResources, setEditResources] = useState<Resource[]>([]);
  const [addDialogOpen, setAddDialogOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<Resource[]>([]);
  const [searching, setSearching] = useState(false);

  const { data: collections, isLoading } = useQuery<Collection[]>({
    queryKey: ['collections'],
    queryFn: collectionApi.list,
  });

  const createMut = useMutation({
    mutationFn: (payload: CreateCollection) => collectionApi.create(payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['collections'] });
      setEditOpen(false);
    },
    onError: (e: Error) => setError(e.message || '创建失败'),
  });

  const updateMut = useMutation({
    mutationFn: ({ id, payload }: { id: number; payload: CreateCollection }) =>
      collectionApi.update(id, payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['collections'] });
      setEditOpen(false);
    },
    onError: (e: Error) => setError(e.message || '更新失败'),
    });

  const deleteMut = useMutation({
    mutationFn: (id: number) => collectionApi.delete(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['collections'] }),
  });

  const openCreate = () => {
    setEditState(emptyEdit);
    setError('');
    setEditOpen(true);
  };

  const openEdit = async (c: Collection) => {
    setEditState({
      id: c.id,
      title: c.title,
      description: c.description,
      visibility: c.visibility,
      accessPassword: c.access_password,
      thumbnailPath: c.thumbnail_path,
    });
    setError('');
    setEditResources([]);
    setEditOpen(true);
    // 拉取合集内文章
    try {
      const result = await resourceApi.list({ page: 1, page_size: 1000, collection_id: c.id });
      setEditResources(result.items);
    } catch (e) {
      console.error('Failed to fetch collection resources:', e);
    }
  };

  const handleRemoveResource = async (rid: number) => {
    if (editState.id == null) return;
    try {
      const newIds = editResources.filter((r) => r.id !== rid).map((r) => r.id);
      await collectionApi.syncResources(editState.id, newIds);
      setEditResources((prev) => prev.filter((r) => r.id !== rid));
      queryClient.invalidateQueries({ queryKey: ['collections'] });
    } catch (e: any) {
      setError(e.message || '移除失败');
    }
  };

  const handleAddResource = async (rid: number) => {
    if (editState.id == null) return;
    try {
      const newIds = [...editResources.map((r) => r.id), rid];
      await collectionApi.syncResources(editState.id, newIds);
      // 刷新合集内文章
      const result = await resourceApi.list({ page: 1, page_size: 1000, collection_id: editState.id });
      setEditResources(result.items);
      // 从搜索结果中移除已添加项
      setSearchResults((prev) => prev.filter((r) => r.id !== rid));
      queryClient.invalidateQueries({ queryKey: ['collections'] });
    } catch (e: any) {
      setError(e.message || '添加失败');
    }
  };

  const handleSearch = useCallback(
    async (query: string) => {
      setSearchQuery(query);
      if (!query.trim()) {
        setSearchResults([]);
        return;
      }
      setSearching(true);
      try {
        const result = await resourceApi.list({ page: 1, page_size: 50, search: query });
        // 过滤掉已经在合集中的资源
        const existingIds = new Set(editResources.map((r) => r.id));
        setSearchResults(result.items.filter((r) => !existingIds.has(r.id)));
      } catch (e) {
        console.error('Search failed:', e);
      } finally {
        setSearching(false);
      }
    },
    [editResources]
  );

  const handleSave = async () => {
    const title = editState.title.trim();
    if (!title) {
      setError('请填写合集名称');
      return;
    }
    setError('');
    const payload: CreateCollection = {
      title,
      description: editState.description.trim(),
      visibility: editState.visibility,
      access_password: editState.accessPassword,
      thumbnail_path: editState.thumbnailPath,
    };
    if (editState.id != null) {
      updateMut.mutate({ id: editState.id, payload });
    } else {
      createMut.mutate(payload);
    }
  };

  const handleDelete = (c: Collection) => {
    if (confirm(`确认删除合集「${c.title}」？关联资源不会被删除。`)) {
      deleteMut.mutate(c.id);
    }
  };

  const batchDelete = () =>
    runBatchAction(
      selection,
      (n) => `确认删除选中的 ${n} 个合集？关联资源不会被删除。`,
      (id) => collectionApi.delete(id),
      () => queryClient.invalidateQueries({ queryKey: ['collections'] }),
    );

  const uploadThumbnail = async () => {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp'] }],
    });
    const filePath = typeof selected === 'string' ? selected : null;
    if (!filePath) return;
    try {
      if (editState.id != null) {
        const result = await imageApi.uploadCollectionImage(editState.id, filePath);
        setEditState((s) => ({ ...s, thumbnailPath: result.file_path }));
      } else {
        // 新建模式下直接保存后上传
        const title = editState.title.trim();
        if (!title) {
          setError('请先填写合集名称再上传缩略图');
          return;
        }
        const payload: CreateCollection = {
          title,
          description: editState.description.trim(),
          visibility: editState.visibility,
          access_password: editState.accessPassword,
          thumbnail_path: '',
        };
        const newId = await collectionApi.create(payload);
        const result = await imageApi.uploadCollectionImage(newId, filePath);
        setEditState((s) => ({ ...s, id: newId, thumbnailPath: result.file_path }));
        queryClient.invalidateQueries({ queryKey: ['collections'] });
      }
    } catch (e: any) {
      setError(e.message || '上传失败');
    }
  };

  const clearThumbnail = () => {
    setEditState((s) => ({ ...s, thumbnailPath: '' }));
  };

  const viewResources = (c: Collection) => {
    navigate(`/?collection=${c.id}`);
  };

  const pending = createMut.isPending || updateMut.isPending;

  return (
    <div className={s.pageContainer}>
      <div className={s.pageHeader}>
        <h2 style={{ margin: 0 }}>合集管理</h2>
        {canCollection && (
          <Button
            icon={<AddRegular />}
            appearance="primary"
            onClick={openCreate}
          >
            新建合集
          </Button>
        )}
      </div>

      {isLoading ? (
        <Spinner label="加载中..." />
      ) : !collections || collections.length === 0 ? (
        <div className={s.emptyState}>
          <p>暂无合集，点击右上角「新建合集」创建</p>
        </div>
      ) : (
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '12px' }}>
          {collections.map((c) => (
            <CardContextMenu
              key={c.id}
              items={canCollection ? [
                {
                  icon: <RenameRegular />,
                  label: '编辑',
                  onClick: () => openEdit(c),
                },
                {
                  icon: <DeleteRegular />,
                  label: '删除',
                  danger: true,
                  onClick: () => handleDelete(c),
                },
              ] : []}
            >
              <div
                className="batch-card"
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '8px',
                  padding: '12px',
                  borderRadius: '8px',
                  border: `1px solid ${tokens.colorNeutralStroke2}`,
                  backgroundColor: tokens.colorNeutralBackground1,
                  position: 'relative',
                  cursor: 'pointer',
                }}
                onClick={() => viewResources(c)}
              >
                {canCollection && (
                  <CardCheckbox
                    checked={selection.selected.has(c.id)}
                    onChange={(checked) => selection.toggle(c.id, checked)}
                  />
                )}
                {c.thumbnail_path ? (
                  <img
                    style={{
                      width: '100%',
                      aspectRatio: '16 / 9',
                      objectFit: 'cover',
                      borderRadius: '6px',
                      backgroundColor: tokens.colorNeutralBackground3,
                    }}
                    src={assetUrl(c.thumbnail_path)}
                    alt={c.title}
                    loading="lazy"
                  />
                ) : (
                  <div
                    style={{
                      width: '100%',
                      aspectRatio: '16 / 9',
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'center',
                      borderRadius: '6px',
                      backgroundColor: tokens.colorNeutralBackground3,
                      fontSize: '32px',
                      color: tokens.colorNeutralForeground3,
                    }}
                  >
                    {c.title.slice(0, 1)}
                  </div>
                )}
                <div className={s.cardBody} style={{ flex: 1 }}>
                  <span className={s.cardTitle}>{c.title}</span>
                  {c.description && <span className={s.cardDesc}>{c.description}</span>}
                  <div style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
                    <Badge appearance="filled" color="informative">
                      {c.resource_count ?? 0} 资源
                    </Badge>
                    <Badge appearance="outline">
                      {c.visibility === 'public' ? '公开' : '私有'}
                    </Badge>
                  </div>
                </div>
              </div>
            </CardContextMenu>
          ))}
        </div>
      )}

      {/* 新建/编辑对话框 */}
      <Dialog open={editOpen} onOpenChange={(_, e: DialogOpenChangeData) => setEditOpen(e.open)}>
        <DialogSurface>
          <DialogBody>
            <DialogTitle>{editState.id != null ? '编辑合集' : '新建合集'}</DialogTitle>
            <DialogContent>
              {error && (
                <div style={{ color: tokens.colorPaletteRedForeground1, marginBottom: '8px' }}>
                  {error}
                </div>
              )}
              <div className={s.field}>
                <Label htmlFor="col-title">名称 *</Label>
                <Input
                  id="col-title"
                  value={editState.title}
                  onChange={(_, d) => setEditState((s) => ({ ...s, title: d.value }))}
                  placeholder="合集名称"
                />
              </div>
              <div className={s.field}>
                <Label htmlFor="col-desc">描述</Label>
                <Textarea
                  id="col-desc"
                  value={editState.description}
                  onChange={(_, d) => setEditState((s) => ({ ...s, description: d.value }))}
                  rows={3}
                  placeholder="可选描述"
                />
              </div>
              <div className={s.field}>
                <Label>可见性</Label>
                <Dropdown
                  value={editState.visibility === 'public' ? '公开' : '私有'}
                  selectedOptions={[editState.visibility]}
                  onOptionSelect={(_, d) => {
                    if (d.optionValue) setEditState((s) => ({ ...s, visibility: d.optionValue! }));
                  }}
                >
                  <Option value="private" text="私有">私有</Option>
                  <Option value="public" text="公开">公开</Option>
                </Dropdown>
              </div>
              <div className={s.field}>
                <Label htmlFor="col-pwd">访问密码（留空则不限制）</Label>
                <Input
                  id="col-pwd"
                  type="password"
                  value={editState.accessPassword}
                  onChange={(_, d) => setEditState((s) => ({ ...s, accessPassword: d.value }))}
                />
              </div>
              <div className={s.field} style={{ marginBottom: 0 }}>
                <Label>缩略图</Label>
                <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
                  {editState.thumbnailPath ? (
                    <img
                      style={{
                        width: '120px',
                        height: '90px',
                        objectFit: 'cover',
                        borderRadius: '6px',
                        backgroundColor: tokens.colorNeutralBackground3,
                      }}
                      src={assetUrl(editState.thumbnailPath)}
                      alt="缩略图"
                    />
                  ) : (
                    <div
                      style={{
                        width: '120px',
                        height: '90px',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        borderRadius: '6px',
                        backgroundColor: tokens.colorNeutralBackground3,
                        color: tokens.colorNeutralForeground3,
                      }}
                    >
                      无
                    </div>
                  )}
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                    <Button
                      appearance="outline"
                      size="small"
                      icon={<ArrowUploadRegular />}
                      onClick={uploadThumbnail}
                    >
                      上传
                    </Button>
                    {editState.thumbnailPath && (
                      <Button
                        appearance="subtle"
                        size="small"
                        icon={<DeleteRegular />}
                        onClick={clearThumbnail}
                      >
                        清除
                      </Button>
                    )}
                  </div>
                </div>
              </div>
              {editState.id != null && (
                <div className={s.field} style={{ marginBottom: 0, marginTop: '12px' }}>
                  <Label>合集内文章（{editResources.length}）</Label>
                  {editResources.length === 0 ? (
                    <div style={{ color: tokens.colorNeutralForeground3, fontSize: '13px' }}>
                      暂无文章
                    </div>
                  ) : (
                    <div style={{
                      display: 'flex', flexDirection: 'column', gap: '6px',
                      maxHeight: '280px', overflowY: 'auto', padding: '4px',
                      border: `1px solid ${tokens.colorNeutralStroke2}`, borderRadius: '6px',
                    }}>
                      {editResources.map((r) => (
                        <div
                          key={r.id}
                          style={{
                            display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                            gap: '8px', padding: '6px 8px', borderRadius: '4px',
                            backgroundColor: tokens.colorNeutralBackground2,
                          }}
                        >
                          <span style={{
                            fontSize: '13px', overflow: 'hidden', textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap', flex: 1,
                          }}>{r.title}</span>
                          <Button
                            size="small"
                            appearance="subtle"
                            icon={<DeleteRegular />}
                            onClick={() => handleRemoveResource(r.id)}
                            aria-label="移除"
                          />
                        </div>
                      ))}
                    </div>
                  )}
                  <div style={{ display: 'flex', justifyContent: 'center', marginTop: '8px' }}>
                    <Button
                      appearance="outline"
                      size="small"
                      icon={<AddRegular />}
                      onClick={() => {
                        setSearchQuery('');
                        setSearchResults([]);
                        setAddDialogOpen(true);
                      }}
                    >
                      添加文章
                    </Button>
                  </div>
                </div>
              )}
            </DialogContent>
            <DialogActions>
              <DialogTrigger>
                <Button appearance="secondary">取消</Button>
              </DialogTrigger>
              <Button
                appearance="primary"
                onClick={handleSave}
                disabled={pending || !editState.title.trim()}
              >
                {pending ? '保存中...' : '保存'}
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>

      {/* 添加文章对话框 */}
      <Dialog
        open={addDialogOpen}
        onOpenChange={(_, e: DialogOpenChangeData) => setAddDialogOpen(e.open)}
      >
        <DialogSurface>
          <DialogBody>
            <DialogTitle>添加文章到合集</DialogTitle>
            <DialogContent>
              <Input
                placeholder="搜索文章标题..."
                value={searchQuery}
                onChange={(_, d) => handleSearch(d.value)}
                contentBefore={<SearchRegular />}
                style={{ width: '100%' }}
              />
              {searching && (
                <div style={{ textAlign: 'center', padding: '12px' }}>
                  <Spinner size="tiny" />
                </div>
              )}
              {!searching && searchQuery.trim() && searchResults.length === 0 && (
                <div
                  style={{
                    textAlign: 'center',
                    padding: '12px',
                    color: tokens.colorNeutralForeground3,
                    fontSize: '13px',
                  }}
                >
                  未找到匹配的文章
                </div>
              )}
              {!searching && searchResults.length > 0 && (
                <div style={{
                  display: 'flex', flexDirection: 'column', gap: '6px',
                  maxHeight: '360px', overflowY: 'auto', marginTop: '8px',
                }}>
                  {searchResults.map((r) => (
                    <div
                      key={r.id}
                      style={{
                        display: 'flex', alignItems: 'center', justifyContent: 'space-between',
                        gap: '8px', padding: '6px 8px', borderRadius: '4px',
                        border: `1px solid ${tokens.colorNeutralStroke2}`,
                      }}
                    >
                      <span style={{
                        fontSize: '13px', overflow: 'hidden', textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap', flex: 1,
                      }}>{r.title}</span>
                      <Button
                        size="small"
                        appearance="primary"
                        icon={<AddRegular />}
                        onClick={() => handleAddResource(r.id)}
                      >
                        添加
                      </Button>
                    </div>
                  ))}
                </div>
              )}
              {!searching && !searchQuery.trim() && (
                <div
                  style={{
                    textAlign: 'center',
                    padding: '24px 12px',
                    color: tokens.colorNeutralForeground3,
                    fontSize: '13px',
                  }}
                >
                  输入关键词搜索文章
                </div>
              )}
            </DialogContent>
            <DialogActions>
              <DialogTrigger>
                <Button appearance="secondary">关闭</Button>
              </DialogTrigger>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>

      {/* 浮动批量操作栏（选中至少一个合集后显示，需合集管理权限） */}
      {canCollection && (
        <BatchBar
          selectedCount={selection.size}
          allChecked={!!collections && collections.length > 0 && collections.every((c) => selection.selected.has(c.id))}
          total={collections?.length ?? 0}
          onSelectAll={(c) => selection.selectAll((collections ?? []).map((x) => x.id), c)}
          onInvert={() => selection.invert((collections ?? []).map((x) => x.id))}
          onClear={selection.clear}
        >
          <Button
            appearance="primary"
            size="small"
            icon={<DeleteRegular />}
            onClick={batchDelete}
          >
            批量删除
          </Button>
        </BatchBar>
      )}
    </div>
  );
}
