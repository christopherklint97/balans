import { createFileRoute } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { adminApi, companiesApi } from '@/api/queries';
import { useAuth } from '@/auth/context';
import type { AdminUser, CompanyUser, Company } from '@/api/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

export const Route = createFileRoute('/admin')({
  component: AdminPage,
});

function AdminPage() {
  const { user } = useAuth();

  if (user?.role !== 'admin') {
    return <p className="text-destructive">Administratörsbehörighet krävs.</p>;
  }

  return (
    <div className="space-y-8">
      <h1 className="text-2xl font-semibold">Administration</h1>
      <PendingApprovals />
      <AllUsers />
      <CompanyUsers />
    </div>
  );
}

function PendingApprovals() {
  const queryClient = useQueryClient();
  const { data: pending, isLoading } = useQuery({
    queryKey: ['admin', 'pending-users'],
    queryFn: adminApi.listPendingUsers,
  });

  const approveMutation = useMutation({
    mutationFn: (id: string) => adminApi.approveUser(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin'] });
    },
  });

  const rejectMutation = useMutation({
    mutationFn: (id: string) => adminApi.rejectUser(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin'] });
    },
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar...</p>;
  if (!pending?.length) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">
          Väntande registreringar
          <Badge variant="destructive" className="ml-2">
            {pending.length}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Namn</TableHead>
              <TableHead>E-post</TableHead>
              <TableHead>Registrerad</TableHead>
              <TableHead className="text-right">Åtgärder</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {pending.map((u) => (
              <TableRow key={u.id}>
                <TableCell>{u.name}</TableCell>
                <TableCell>{u.email}</TableCell>
                <TableCell className="text-muted-foreground">
                  {new Date(u.created_at).toLocaleDateString('sv-SE')}
                </TableCell>
                <TableCell className="text-right space-x-2">
                  <Button
                    size="sm"
                    onClick={() => approveMutation.mutate(u.id)}
                    disabled={approveMutation.isPending}
                  >
                    Godkänn
                  </Button>
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => rejectMutation.mutate(u.id)}
                    disabled={rejectMutation.isPending}
                  >
                    Neka
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

function AllUsers() {
  const queryClient = useQueryClient();
  const { data: users, isLoading } = useQuery({
    queryKey: ['admin', 'users'],
    queryFn: adminApi.listUsers,
  });

  const roleMutation = useMutation({
    mutationFn: ({ id, role }: { id: string; role: string }) =>
      adminApi.changeUserRole(id, role),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin'] });
    },
  });

  const toggleActiveMutation = useMutation({
    mutationFn: (user: AdminUser) =>
      user.is_active ? adminApi.deactivateUser(user.id) : adminApi.activateUser(user.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin'] });
    },
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar...</p>;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Alla användare</CardTitle>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Namn</TableHead>
              <TableHead>E-post</TableHead>
              <TableHead>Roll</TableHead>
              <TableHead>Status</TableHead>
              <TableHead className="text-right">Åtgärder</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {users?.map((u) => (
              <TableRow key={u.id}>
                <TableCell>{u.name}</TableCell>
                <TableCell>{u.email}</TableCell>
                <TableCell>
                  <select
                    value={u.role}
                    onChange={(e) =>
                      roleMutation.mutate({ id: u.id, role: e.target.value })
                    }
                    className="text-sm border rounded px-2 py-1"
                  >
                    <option value="admin">Admin</option>
                    <option value="user">User</option>
                    <option value="viewer">Viewer</option>
                  </select>
                </TableCell>
                <TableCell>
                  <StatusBadge status={u.status} isActive={u.is_active} />
                </TableCell>
                <TableCell className="text-right">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => toggleActiveMutation.mutate(u)}
                    disabled={toggleActiveMutation.isPending}
                  >
                    {u.is_active ? 'Inaktivera' : 'Aktivera'}
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}

function CompanyUsers() {
  const { data: companies } = useQuery({
    queryKey: ['companies'],
    queryFn: companiesApi.list,
  });

  const [selectedCompany, setSelectedCompany] = useState<string>('');

  if (!companies?.length) return null;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Företagsanvändare</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <select
          value={selectedCompany}
          onChange={(e) => setSelectedCompany(e.target.value)}
          className="text-sm border rounded px-3 py-2 w-full max-w-md"
        >
          <option value="">Välj företag...</option>
          {companies.map((c: Company) => (
            <option key={c.id} value={c.id}>
              {c.name} ({c.org_number})
            </option>
          ))}
        </select>

        {selectedCompany && <CompanyUserList companyId={selectedCompany} />}
      </CardContent>
    </Card>
  );
}

function CompanyUserList({ companyId }: { companyId: string }) {
  const queryClient = useQueryClient();

  const { data: users, isLoading } = useQuery({
    queryKey: ['admin', 'company-users', companyId],
    queryFn: () => adminApi.listCompanyUsers(companyId),
  });

  const { data: allUsers } = useQuery({
    queryKey: ['admin', 'users'],
    queryFn: adminApi.listUsers,
  });

  const [addUserId, setAddUserId] = useState('');
  const [addRole, setAddRole] = useState('member');

  const roleMutation = useMutation({
    mutationFn: ({ userId, role }: { userId: string; role: string }) =>
      adminApi.changeCompanyRole(companyId, userId, role),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'company-users', companyId] });
    },
  });

  const removeMutation = useMutation({
    mutationFn: (userId: string) => adminApi.removeCompanyUser(companyId, userId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'company-users', companyId] });
    },
  });

  const addMutation = useMutation({
    mutationFn: () => adminApi.addCompanyUser(companyId, addUserId, addRole),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['admin', 'company-users', companyId] });
      setAddUserId('');
    },
  });

  if (isLoading) return <p className="text-muted-foreground">Laddar...</p>;

  const existingUserIds = new Set(users?.map((u: CompanyUser) => u.user_id) ?? []);
  const availableUsers = allUsers?.filter((u: AdminUser) => !existingUserIds.has(u.id)) ?? [];

  return (
    <div className="space-y-4">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Namn</TableHead>
            <TableHead>E-post</TableHead>
            <TableHead>Företagsroll</TableHead>
            <TableHead className="text-right">Åtgärder</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {users?.map((u: CompanyUser) => (
            <TableRow key={u.user_id}>
              <TableCell>{u.name}</TableCell>
              <TableCell>{u.email}</TableCell>
              <TableCell>
                <select
                  value={u.company_role}
                  onChange={(e) =>
                    roleMutation.mutate({ userId: u.user_id, role: e.target.value })
                  }
                  className="text-sm border rounded px-2 py-1"
                >
                  <option value="owner">Owner</option>
                  <option value="admin">Admin</option>
                  <option value="member">Member</option>
                  <option value="viewer">Viewer</option>
                </select>
              </TableCell>
              <TableCell className="text-right">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => removeMutation.mutate(u.user_id)}
                  disabled={removeMutation.isPending}
                >
                  Ta bort
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      {availableUsers.length > 0 && (
        <div className="flex items-end gap-2">
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">Lägg till användare</label>
            <select
              value={addUserId}
              onChange={(e) => setAddUserId(e.target.value)}
              className="text-sm border rounded px-3 py-2"
            >
              <option value="">Välj användare...</option>
              {availableUsers.map((u: AdminUser) => (
                <option key={u.id} value={u.id}>
                  {u.name} ({u.email})
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">Roll</label>
            <select
              value={addRole}
              onChange={(e) => setAddRole(e.target.value)}
              className="text-sm border rounded px-3 py-2"
            >
              <option value="owner">Owner</option>
              <option value="admin">Admin</option>
              <option value="member">Member</option>
              <option value="viewer">Viewer</option>
            </select>
          </div>
          <Button
            size="sm"
            onClick={() => addMutation.mutate()}
            disabled={!addUserId || addMutation.isPending}
          >
            Lägg till
          </Button>
        </div>
      )}
    </div>
  );
}

function StatusBadge({ status, isActive }: { status: string; isActive: boolean }) {
  if (!isActive) {
    return <Badge variant="secondary">Inaktiv</Badge>;
  }
  switch (status) {
    case 'approved':
      return <Badge variant="default">Aktiv</Badge>;
    case 'pending':
      return <Badge variant="outline">Väntande</Badge>;
    case 'rejected':
      return <Badge variant="destructive">Nekad</Badge>;
    default:
      return <Badge variant="secondary">{status}</Badge>;
  }
}
