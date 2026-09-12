// 登录页
import { useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router-dom';
import {
  Card,
  CardHeader,
  Input,
  Button,
  Label,
  makeStyles,
} from '@fluentui/react-components';
import { useAuthStore } from '../stores/authStore';

const useStyles = makeStyles({
  container: {
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'center',
    minHeight: '100vh',
  },
  card: {
    width: '400px',
    padding: '24px',
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
    marginBottom: '16px',
  },
  footer: {
    marginTop: '12px',
    textAlign: 'center',
    fontSize: '13px',
  },
});

export function LoginPage() {
  const styles = useStyles();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const next = searchParams.get('next') || '/';
  const { login } = useAuthStore();
  const [username, setUsername] = useState('admin');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError('');
    try {
      await login(username, password);
      navigate(next);
    } catch (err: any) {
      setError(err.message || '登录失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.container}>
      <Card className={styles.card}>
        <CardHeader header={<h2>登录</h2>} />
        <form onSubmit={handleSubmit}>
          <div className={styles.field}>
            <Label htmlFor="username">用户名</Label>
            <Input
              id="username"
              value={username}
              onChange={(_, d) => setUsername(d.value)}
              required
            />
          </div>
          <div className={styles.field}>
            <Label htmlFor="password">密码</Label>
            <Input
              id="password"
              type="password"
              value={password}
              onChange={(_, d) => setPassword(d.value)}
              required
            />
          </div>
          {error && <div style={{ color: 'red', marginBottom: '8px' }}>{error}</div>}
          <Button type="submit" appearance="primary" disabled={loading} style={{ width: '100%' }}>
            {loading ? '登录中...' : '登录'}
          </Button>
        </form>
        <div className={styles.footer}>
          还没有账户？<Link to="/register">注册</Link>
        </div>
      </Card>
    </div>
  );
}
