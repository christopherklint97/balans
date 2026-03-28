import { createRootRoute, Link, Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { useState, useEffect } from 'react';
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
  const [menuOpen, setMenuOpen] = useState(false);

  // Close mobile menu on route change
  useEffect(() => {
    setMenuOpen(false);
  }, [location.pathname]);

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
        <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-4">
          <Link to="/" className="text-lg font-semibold tracking-tight">
            Balans
          </Link>

          {/* Desktop nav */}
          <div className="hidden md:flex items-center gap-4 text-sm">
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

          <div className="hidden md:flex items-center gap-3">
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

          {/* Mobile hamburger */}
          <button
            className="md:hidden flex flex-col gap-1.5 p-2 -mr-2"
            onClick={() => setMenuOpen(!menuOpen)}
            aria-label="Meny"
          >
            <span className={`block h-0.5 w-5 bg-foreground transition-transform ${menuOpen ? 'translate-y-2 rotate-45' : ''}`} />
            <span className={`block h-0.5 w-5 bg-foreground transition-opacity ${menuOpen ? 'opacity-0' : ''}`} />
            <span className={`block h-0.5 w-5 bg-foreground transition-transform ${menuOpen ? '-translate-y-2 -rotate-45' : ''}`} />
          </button>
        </div>

        {/* Mobile menu */}
        {menuOpen && (
          <div className="md:hidden border-t border-border bg-background px-4 py-3 space-y-1">
            <MobileNavLink to="/">Kontrollpanel</MobileNavLink>
            <MobileNavLink to="/accounts">Kontoplan</MobileNavLink>
            <MobileNavLink to="/assets">Tillgångar</MobileNavLink>
            <MobileNavLink to="/vouchers">Verifikationer</MobileNavLink>
            <MobileNavLink to="/reports">Rapporter</MobileNavLink>
            <MobileNavLink to="/closing">Bokslut</MobileNavLink>
            <MobileNavLink to="/sie">SIE</MobileNavLink>
            <MobileNavLink to="/tax">INK2</MobileNavLink>
            <MobileNavLink to="/filing">Inlämning</MobileNavLink>
            <MobileNavLink to="/compliance">Compliance</MobileNavLink>
            {user.role === 'admin' && (
              <div className="py-2">
                <AdminNavLink />
              </div>
            )}
            <div className="pt-2 border-t border-border flex items-center justify-between">
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
        )}
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

function MobileNavLink({ to, children }: { to: string; children: React.ReactNode }) {
  return (
    <Link
      to={to}
      className="block py-2 text-sm text-muted-foreground hover:text-foreground transition-colors [&.active]:text-foreground [&.active]:font-medium"
    >
      {children}
    </Link>
  );
}
