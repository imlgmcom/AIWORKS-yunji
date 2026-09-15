// 作者列表页：展示有文章的作者，点击进入该作者的文章列表
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { Avatar, Badge, Spinner, Input, makeStyles, tokens } from '@fluentui/react-components';
import { PeopleRegular, SearchRegular } from '@fluentui/react-icons';
import { authorApi, assetUrl } from '../lib/tauri';
import { Pagination } from '../components/Pagination';
import { useSharedStyles } from '../styles/shared';

const PAGE_SIZE = 36;

const useStyles = makeStyles({
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(220px, 1fr))',
    gap: '12px',
  },
  authorCard: {
    display: 'flex',
    alignItems: 'center',
    gap: '12px',
    padding: '14px 16px',
    borderRadius: '8px',
    border: `1px solid ${tokens.colorNeutralStroke2}`,
    backgroundColor: tokens.colorNeutralBackground1,
    cursor: 'pointer',
    transition: 'background 0.15s',
    '&:hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
    },
  },
  bio: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground3,
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
    marginTop: '2px',
  },
});

export function AuthorsPage() {
  const s = useSharedStyles();
  const styles = useStyles();
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [keyword, setKeyword] = useState('');

  const { data, isLoading } = useQuery({
    queryKey: ['authors', 'paged', page, keyword],
    queryFn: () => authorApi.listPage(page, PAGE_SIZE, keyword || undefined),
  });

  const authors = data?.items ?? [];
  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <div className={s.pageContainer}>
      <div className={s.pageHeader}>
        <h2 style={{ margin: 0 }}>作者</h2>
        <Input
          style={{ width: '240px' }}
          placeholder="搜索作者"
          value={keyword}
          onChange={(_, d) => {
            setKeyword(d.value);
            setPage(1);
          }}
          contentBefore={<SearchRegular />}
        />
      </div>

      {isLoading ? (
        <Spinner label="加载中..." />
      ) : authors.length === 0 ? (
        <div className={s.emptyState}>
          <PeopleRegular fontSize={36} />
          <p>{keyword ? '没有匹配的作者' : '暂无作者'}</p>
        </div>
      ) : (
        <>
          <div className={styles.grid}>
            {authors.map((a) => {
              const name = a.nickname || a.username;
              return (
                <div
                  key={a.id}
                  className={styles.authorCard}
                  onClick={() => navigate(`/?author=${a.id}`)}
                  title={`查看 ${name} 的全部文章`}
                >
                  <Avatar
                    size={48}
                    name={name}
                    image={a.avatar_path ? { src: assetUrl(a.avatar_path) } : undefined}
                  />
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                      <strong style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                        {name}
                      </strong>
                      <Badge appearance="tint" color="brand">{a.resource_count} 篇</Badge>
                    </div>
                    {a.bio && <div className={styles.bio}>{a.bio}</div>}
                  </div>
                </div>
              );
            })}
          </div>
          <Pagination page={page} totalPages={totalPages} total={total} onChange={setPage} />
        </>
      )}
    </div>
  );
}
