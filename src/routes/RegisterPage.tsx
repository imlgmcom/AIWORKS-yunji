// 注册页
import { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import {
  Card,
  CardHeader,
  Input,
  Button,
  Label,
  makeStyles,
} from '@fluentui/react-components';
import { useAuthStore } from '../stores/authStore';
import { authApi } from '../lib/tauri';

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

export function RegisterPage() {
  const styles = useStyles();
  const navigate = useNavigate();
  const { setUser } = useAuthStore();
  const [username, setUsername] = useState('');
  const [nickname, setNickname] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    if (password !== confirmPassword) {
      setError('两次输入的密码不一致');
      return;
    }
    setLoading(true);
    try {
      const user = await authApi.register(username.trim(), nickname.trim(), email.trim(), password);
      setUser(user);
      navigate('/');
    } catch (err: any) {
      setError(err.message || '注册失败');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.container}>
      <Card className={styles.card}>
        <CardHeader header={<h2>注册</h2>} />
        <form onSubmit={handleSubmit}>
          <div className={styles.field}>
            <Label htmlFor="username">用户名</Label>
            <Input
              id="username"
              value={username}
              onChange={(_, d) => setUsername(d.value)}
              placeholder="2-32 个字符"
              required
            />
          </div>
          <div className={styles.field}>
            <Label htmlFor="nickname">昵称</Label>
            <Input
              id="nickname"
              value={nickname}
              onChange={(_, d) => setNickname(d.value)}
              required
            />
          </div>
          <div className={styles.field}>
            <Label htmlFor="email">邮箱</Label>
            <Input
              id="email"
              type="email"
              value={email}
              onChange={(_, d) => setEmail(d.value)}
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
              placeholder="至少 6 位"
              required
            />
          </div>
          <div className={styles.field}>
            <Label htmlFor="confirm">确认密码</Label>
            <Input
              id="confirm"
              type="password"
              value={confirmPassword}
              onChange={(_, d) => setConfirmPassword(d.value)}
              required
            />
          </div>
          {error && <div style={{ color: 'red', marginBottom: '8px' }}>{error}</div>}
          <Button type="submit" appearance="primary" disabled={loading} style={{ width: '100%' }}>
            {loading ? '注册中...' : '注册'}
          </Button>
        </form>
        <div className={styles.footer}>
          已有账户？<Link to="/login">去登录</Link>
        </div>
      </Card>
    </div>
  );
}
