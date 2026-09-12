// 回收站（瀑布流展示 + 浮动批量操作栏）
import {
  Button,
  Spinner,
  Card,
  tokens,
} from '@fluentui/react-components';
import { ArrowCounterclockwiseRegular, DeleteRegular } from '@fluentui/react-icons';
import { useNavigate } from 'react-router-dom';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { resourceApi } from '../lib/tauri';
import { useUIStore } from '../stores/uiStore';
import type { Resource } from '../types/models';
import { useSelection, BatchBar, runBatchAction } from '../components/batch';
import { ResourceCard } from '../components/ResourceCard';
import { CardContextMenu } from '../components/CardContextMenu';
import { useSharedStyles } from '../styles/shared';

export function TrashPage() {
  const s = useSharedStyles();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { viewMode } = useUIStore();
  const selection = useSelection();

  const { data: resources, isLoading } = useQuery({
    queryKey: ['trash'],
    queryFn: resourceApi.listTrash,
  });

  const restoreMut = useMutation({
    mutationFn: (rid: number) => resourceApi.restore(rid),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['trash'] });
      queryClient.invalidateQueries({ queryKey: ['resources'] });
    },
  });

  const permanentDeleteMut = useMutation({
    mutationFn: (rid: number) => resourceApi.permanentDelete(rid),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['trash'] }),
  });

  const handlePermanentDelete = (r: Resource) => {
    if (confirm(`彻底删除「${r.title}」？此操作不可恢复，关联的所有文件将被清除。`))
      permanentDeleteMut.mutate(r.id);
  };

  const items = resources ?? [];
  const itemIds = items.map((r) => r.id);
  const allChecked = items.length > 0 && items.every((r) => selection.selected.has(r.id));
  const busy = restoreMut.isPending || permanentDeleteMut.isPending;

  return (
    <div className={s.pageContainer}>
      <div className={s.pageHeader}>
        <h2 style={{ margin: 0 }}>回收站</h2>
        <Button appearance="outline" onClick={() => navigate('/')}>返回首页</Button>
      </div>

      {isLoading ? (
        <Spinner label="加载中..." />
      ) : items.length === 0 ? (
        <Card>
          <div style={{ padding: '24px', textAlign: 'center', color: tokens.colorNeutralForeground2 }}>
            回收站为空
          </div>
        </Card>
      ) : (
        <div className={
          viewMode === 'masonry' ? s.masonry
          : viewMode === 'card-v' ? s.gridV
          : viewMode === 'card-h' ? s.gridH
          : s.tableList
        }>
          {items.map((r) => (
            <CardContextMenu
              key={r.id}
              items={[
                {
                  icon: <ArrowCounterclockwiseRegular />,
                  label: '恢复',
                  onClick: () => restoreMut.mutate(r.id),
                },
                {
                  icon: <DeleteRegular />,
                  label: '彻底删除',
                  danger: true,
                  onClick: () => handlePermanentDelete(r),
                },
              ]}
            >
              <ResourceCard
                resource={r}
                checked={selection.selected.has(r.id)}
                onToggle={(c) => selection.toggle(r.id, c)}
                showDescription={false}
                viewMode={viewMode}
              />
            </CardContextMenu>
          ))}
        </div>
      )}

      {/* 浮动批量操作栏 */}
      <BatchBar
        selectedCount={selection.size}
        allChecked={allChecked}
        total={items.length}
        onSelectAll={(c) => selection.selectAll(itemIds, c)}
        onInvert={() => selection.invert(itemIds)}
        onClear={selection.clear}
        busy={busy}
      >
        <Button
          appearance="outline"
          size="small"
          icon={<ArrowCounterclockwiseRegular />}
          onClick={() =>
            runBatchAction(
              selection,
              (n) => `将选中的 ${n} 项恢复？`,
              (id) => restoreMut.mutateAsync(id),
              () => queryClient.invalidateQueries({ queryKey: ['trash'] }),
            )
          }
          disabled={busy}
        >
          批量恢复
        </Button>
        <Button
          appearance="primary"
          size="small"
          icon={<DeleteRegular />}
          onClick={() =>
            runBatchAction(
              selection,
              (n) => `将选中的 ${n} 项彻底删除？此操作不可恢复。`,
              (id) => permanentDeleteMut.mutateAsync(id),
              () => queryClient.invalidateQueries({ queryKey: ['trash'] }),
            )
          }
          disabled={busy}
        >
          批量彻底删除
        </Button>
      </BatchBar>
    </div>
  );
}
