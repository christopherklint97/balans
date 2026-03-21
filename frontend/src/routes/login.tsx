import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { authApi } from '@/api/queries';
import { useAuth } from '@/auth/context';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

export const Route = createFileRoute('/login')({
  component: LoginPage,
});

function LoginPage() {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [isRegister, setIsRegister] = useState(false);
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [name, setName] = useState('');
  const [error, setError] = useState('');
  const [pendingMessage, setPendingMessage] = useState('');

  const loginMutation = useMutation({
    mutationFn: () => authApi.login({ email, password }),
    onSuccess: (data) => {
      if (data.status === 'pending') {
        setPendingMessage(data.message || 'Ditt konto väntar på godkännande.');
        return;
      }
      if (data.token && data.user) {
        login(data.token, data.user);
        navigate({ to: '/' });
      }
    },
    onError: (err: Error) => setError(err.message),
  });

  const registerMutation = useMutation({
    mutationFn: () => authApi.register({ email, password, name }),
    onSuccess: (data) => {
      if (data.status === 'pending') {
        setPendingMessage(
          data.message || 'Registrering mottagen. Väntar på godkännande från administratör.',
        );
        return;
      }
      if (data.token && data.user) {
        login(data.token, data.user);
        navigate({ to: '/' });
      }
    },
    onError: (err: Error) => setError(err.message),
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setPendingMessage('');
    if (isRegister) {
      registerMutation.mutate();
    } else {
      loginMutation.mutate();
    }
  };

  const isPending = loginMutation.isPending || registerMutation.isPending;

  if (pendingMessage) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center px-4">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <p className="text-2xl font-semibold tracking-tight">Balans</p>
            <CardTitle className="text-base font-normal text-muted-foreground">
              Väntar på godkännande
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4 text-center">
            <p className="text-sm text-muted-foreground">{pendingMessage}</p>
            <Button
              variant="outline"
              onClick={() => {
                setPendingMessage('');
                setIsRegister(false);
              }}
            >
              Tillbaka till inloggning
            </Button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background flex items-center justify-center px-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <p className="text-2xl font-semibold tracking-tight">Balans</p>
          <CardTitle className="text-base font-normal text-muted-foreground">
            {isRegister ? 'Skapa konto' : 'Logga in'}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleSubmit} className="space-y-4">
            {isRegister && (
              <div className="space-y-2">
                <Label htmlFor="name">Namn</Label>
                <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Anna Andersson"
                  required={isRegister}
                />
              </div>
            )}
            <div className="space-y-2">
              <Label htmlFor="email">E-post</Label>
              <Input
                id="email"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="anna@foretag.se"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="password">Lösenord</Label>
              <Input
                id="password"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Minst 8 tecken"
                required
                minLength={8}
              />
            </div>

            {error && <p className="text-sm text-destructive">{error}</p>}

            <Button type="submit" className="w-full" disabled={isPending}>
              {isPending ? 'Vänta...' : isRegister ? 'Skapa konto' : 'Logga in'}
            </Button>

            <p className="text-center text-sm text-muted-foreground">
              {isRegister ? 'Har redan konto?' : 'Inget konto?'}{' '}
              <button
                type="button"
                onClick={() => {
                  setIsRegister(!isRegister);
                  setError('');
                }}
                className="underline hover:text-foreground"
              >
                {isRegister ? 'Logga in' : 'Registrera dig'}
              </button>
            </p>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
