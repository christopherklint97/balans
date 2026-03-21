import { createRootRoute, Link, Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { useAuth } from '@/auth/context';
import { adminApi } from '@/api/queries';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  const { user, isLoading, logout } = useAuth();
  const location = useLocation();
  const navigate = useNavigate();

  // Show nothing while checking auth
  if (isLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <p className="text-muted-foreground">Laddar...</p>
      </div>
    );
  }

  // Login page renders without the nav shell
  if (location.pathname === '/login') {
    return <Outlet />;
  }

  // Redirect to login if not authenticated
  if (!user) {
    navigate({ to: '/login' });
    return null;
  }

  return (
    <div className="min-h-screen bg-background text-foreground">
      <nav className="border-b border-border">
        <div className="mx-auto flex h-14 max-w-6xl items-center gap-6 px-4">
          <Link to="/" className="text-lg font-semibold tracking-tight">
            Balans
          </Link>
          <div className="flex gap-4 text-sm">
            <NavLink to="/">Kontrollpanel</NavLink>
            <NavLink to="/accounts">Kontoplan</NavLink>
            <NavLink to="/assets">Tillgångar</NavLink>
            <NavLink to="/vouchers">Verifikationer</NavLink>
            <NavLink to="/reports">Rapporter</NavLink>
            <NavLink to="/closing">Bokslut</NavLink>
            <NavLink to="/sie">SIE</NavLink>
            <NavLink to="/tax">INK2</NavLink>
            <NavLink to="/filing">Inlämning</NavLink>
            <NavLink to="/compliance">Compliance</NavLink>
            {user.role === 'admin' && <AdminNavLink />}
          </div>
          <div className="ml-auto flex items-center gap-3">
            <span className="text-xs text-muted-foreground">{user.name}</span>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                logout();
                navigate({ to: '/login' });
              }}
            >
              Logga ut
            </Button>
          </div>
        </div>
      </nav>
      <main className="mx-auto max-w-6xl px-4 py-6">
        <Outlet />
      </main>
    </div>
  );
}

function AdminNavLink() {
  const { data: pending } = useQuery({
    queryKey: ['admin', 'pending-users'],
    queryFn: adminApi.listPendingUsers,
    refetchInterval: 30000,
  });

  const count = pending?.length ?? 0;

  return (
    <Link
      to="/admin"
      className="text-muted-foreground hover:text-foreground transition-colors [&.active]:text-foreground [&.active]:font-medium flex items-center gap-1"
    >
      Admin
      {count > 0 && (
        <Badge variant="destructive" className="h-5 min-w-5 px-1 text-xs">
          {count}
        </Badge>
      )}
    </Link>
  );
}

function NavLink({ to, children }: { to: string; children: React.ReactNode }) {
  return (
    <Link
      to={to}
      className="text-muted-foreground hover:text-foreground transition-colors [&.active]:text-foreground [&.active]:font-medium"
    >
      {children}
    </Link>
  );
}
