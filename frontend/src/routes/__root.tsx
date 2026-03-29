import { createRootRoute, Link, Outlet, useLocation, useNavigate } from '@tanstack/react-router';
import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';
import { useAuth } from '@/auth/context';
import { useTheme } from '@/hooks/use-theme';
import { adminApi } from '@/api/queries';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Sun, Moon, Monitor, LayoutDashboard, BookOpen, Package, FileText, BarChart3, Lock, FileDown, Calculator, Send, ShieldCheck, Settings, LogOut } from 'lucide-react';

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  const { user, isLoading, logout } = useAuth();
  const { theme, setTheme } = useTheme();
  const location = useLocation();
  const navigate = useNavigate();
  const [menuOpen, setMenuOpen] = useState(false);

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
    <div className="min-h-screen bg-background text-foreground flex">
      {/* Desktop sidebar */}
      <aside className="hidden md:flex flex-col w-56 shrink-0 border-r border-border h-screen sticky top-0">
        <div className="px-4 py-4">
          <Link to="/" className="text-lg font-semibold tracking-tight">
            Balans
          </Link>
        </div>

        <nav className="flex-1 overflow-y-auto px-2 space-y-0.5">
          <SidebarLink to="/" icon={<LayoutDashboard className="h-4 w-4" />}>Kontrollpanel</SidebarLink>
          <SidebarLink to="/accounts" icon={<BookOpen className="h-4 w-4" />}>Kontoplan</SidebarLink>
          <SidebarLink to="/assets" icon={<Package className="h-4 w-4" />}>Tillgångar</SidebarLink>
          <SidebarLink to="/vouchers" icon={<FileText className="h-4 w-4" />}>Verifikationer</SidebarLink>
          <SidebarLink to="/reports" icon={<BarChart3 className="h-4 w-4" />}>Rapporter</SidebarLink>
          <SidebarLink to="/closing" icon={<Lock className="h-4 w-4" />}>Bokslut</SidebarLink>
          <SidebarLink to="/sie" icon={<FileDown className="h-4 w-4" />}>SIE</SidebarLink>
          <SidebarLink to="/tax" icon={<Calculator className="h-4 w-4" />}>INK2</SidebarLink>
          <SidebarLink to="/filing" icon={<Send className="h-4 w-4" />}>Inlämning</SidebarLink>
          <SidebarLink to="/compliance" icon={<ShieldCheck className="h-4 w-4" />}>Compliance</SidebarLink>
          {user.role === 'admin' && <AdminSidebarLink />}
        </nav>

        <div className="border-t border-border px-3 py-3 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground truncate">{user.name}</span>
            <ThemeToggle theme={theme} setTheme={setTheme} />
          </div>
          <Button
            variant="ghost"
            size="sm"
            className="w-full justify-start gap-2 text-muted-foreground"
            onClick={() => {
              logout();
              navigate({ to: '/login' });
            }}
          >
            <LogOut className="h-4 w-4" />
            Logga ut
          </Button>
        </div>
      </aside>

      <div className="flex-1 flex flex-col min-w-0">
        {/* Mobile top bar */}
        <nav className="md:hidden border-b border-border">
          <div className="flex h-14 items-center justify-between px-4">
            <Link to="/" className="text-lg font-semibold tracking-tight">
              Balans
            </Link>
            <button
              className="flex flex-col gap-1.5 p-2 -mr-2"
              onClick={() => setMenuOpen(!menuOpen)}
              aria-label="Meny"
            >
              <span className={`block h-0.5 w-5 bg-foreground transition-transform ${menuOpen ? 'translate-y-2 rotate-45' : ''}`} />
              <span className={`block h-0.5 w-5 bg-foreground transition-opacity ${menuOpen ? 'opacity-0' : ''}`} />
              <span className={`block h-0.5 w-5 bg-foreground transition-transform ${menuOpen ? '-translate-y-2 -rotate-45' : ''}`} />
            </button>
          </div>

          {menuOpen && (
            <div className="border-t border-border bg-background px-4 py-3 space-y-1">
              <MobileNavLink to="/" onClick={() => setMenuOpen(false)}>Kontrollpanel</MobileNavLink>
              <MobileNavLink to="/accounts" onClick={() => setMenuOpen(false)}>Kontoplan</MobileNavLink>
              <MobileNavLink to="/assets" onClick={() => setMenuOpen(false)}>Tillgångar</MobileNavLink>
              <MobileNavLink to="/vouchers" onClick={() => setMenuOpen(false)}>Verifikationer</MobileNavLink>
              <MobileNavLink to="/reports" onClick={() => setMenuOpen(false)}>Rapporter</MobileNavLink>
              <MobileNavLink to="/closing" onClick={() => setMenuOpen(false)}>Bokslut</MobileNavLink>
              <MobileNavLink to="/sie" onClick={() => setMenuOpen(false)}>SIE</MobileNavLink>
              <MobileNavLink to="/tax" onClick={() => setMenuOpen(false)}>INK2</MobileNavLink>
              <MobileNavLink to="/filing" onClick={() => setMenuOpen(false)}>Inlämning</MobileNavLink>
              <MobileNavLink to="/compliance" onClick={() => setMenuOpen(false)}>Compliance</MobileNavLink>
              {user.role === 'admin' && (
                <div className="py-2">
                  <AdminNavLink />
                </div>
              )}
              <div className="pt-2 border-t border-border flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className="text-xs text-muted-foreground">{user.name}</span>
                  <ThemeToggle theme={theme} setTheme={setTheme} />
                </div>
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

        <main className="flex-1 px-4 py-6 md:px-8 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}

function SidebarLink({ to, icon, children }: { to: string; icon: React.ReactNode; children: React.ReactNode }) {
  return (
    <Link
      to={to}
      className="flex items-center gap-3 rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:text-foreground hover:bg-accent transition-colors [&.active]:text-foreground [&.active]:bg-accent [&.active]:font-medium"
    >
      {icon}
      {children}
    </Link>
  );
}

function AdminSidebarLink() {
  const { data: pending } = useQuery({
    queryKey: ['admin', 'pending-users'],
    queryFn: adminApi.listPendingUsers,
    refetchInterval: 30000,
  });

  const count = pending?.length ?? 0;

  return (
    <Link
      to="/admin"
      className="flex items-center gap-3 rounded-md px-2 py-1.5 text-sm text-muted-foreground hover:text-foreground hover:bg-accent transition-colors [&.active]:text-foreground [&.active]:bg-accent [&.active]:font-medium"
    >
      <Settings className="h-4 w-4" />
      Admin
      {count > 0 && (
        <Badge variant="destructive" className="h-5 min-w-5 px-1 text-xs ml-auto">
          {count}
        </Badge>
      )}
    </Link>
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

function MobileNavLink({ to, onClick, children }: { to: string; onClick?: () => void; children: React.ReactNode }) {
  return (
    <Link
      to={to}
      onClick={onClick}
      className="block py-2 text-sm text-muted-foreground hover:text-foreground transition-colors [&.active]:text-foreground [&.active]:font-medium"
    >
      {children}
    </Link>
  );
}

function ThemeToggle({ theme, setTheme }: { theme: string; setTheme: (t: 'light' | 'dark' | 'system') => void }) {
  const next = theme === 'light' ? 'dark' : theme === 'dark' ? 'system' : 'light';
  const icon = theme === 'light' ? <Sun className="h-4 w-4" /> : theme === 'dark' ? <Moon className="h-4 w-4" /> : <Monitor className="h-4 w-4" />;
  const label = theme === 'light' ? 'Ljust läge' : theme === 'dark' ? 'Mörkt läge' : 'Systemläge';

  return (
    <button
      onClick={() => setTheme(next)}
      className="inline-flex items-center justify-center rounded-md p-1.5 text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
      title={label}
      aria-label={label}
    >
      {icon}
    </button>
  );
}
