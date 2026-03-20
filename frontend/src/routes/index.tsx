import { createFileRoute, Link } from '@tanstack/react-router';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import { companiesApi, fiscalYearsApi } from '@/api/queries';
import type { Company } from '@/api/types';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';

export const Route = createFileRoute('/')({
  component: Dashboard,
});

function Dashboard() {
  const queryClient = useQueryClient();
  const [showForm, setShowForm] = useState(false);

  const { data: companies, isLoading } = useQuery({
    queryKey: ['companies'],
    queryFn: companiesApi.list,
  });

  if (isLoading) {
    return <p className="text-muted-foreground">Laddar...</p>;
  }

  if (!companies?.length) {
    return (
      <div className="space-y-6">
        <h1 className="text-2xl font-semibold">Välkommen till Balans</h1>
        <p className="text-muted-foreground">
          Skapa ett företag för att komma igång.
        </p>
        <CreateCompanyForm onSuccess={() => queryClient.invalidateQueries({ queryKey: ['companies'] })} />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-semibold">Kontrollpanel</h1>
        <Button variant="outline" size="sm" onClick={() => setShowForm(!showForm)}>
          {showForm ? 'Avbryt' : 'Nytt företag'}
        </Button>
      </div>

      {showForm && (
        <CreateCompanyForm
          onSuccess={() => {
            queryClient.invalidateQueries({ queryKey: ['companies'] });
            setShowForm(false);
          }}
        />
      )}

      <div className="grid gap-4 md:grid-cols-2">
        {companies.map((company) => (
          <CompanyCard key={company.id} company={company} />
        ))}
      </div>
    </div>
  );
}

function CompanyCard({ company }: { company: Company }) {
  const { data: fiscalYears } = useQuery({
    queryKey: ['fiscal-years', company.id],
    queryFn: () => fiscalYearsApi.list(company.id),
  });

  const activeFy = fiscalYears?.find((fy) => !fy.is_closed);

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">{company.name}</CardTitle>
          <Badge variant="secondary">{company.company_form}</Badge>
        </div>
        <p className="text-sm text-muted-foreground">{company.org_number}</p>
      </CardHeader>
      <CardContent className="space-y-2">
        {activeFy ? (
          <div className="text-sm">
            <span className="text-muted-foreground">Räkenskapsår: </span>
            {activeFy.start_date} — {activeFy.end_date}
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">Inget räkenskapsår skapat</p>
        )}
        <div className="flex gap-2 pt-2">
          <Link to="/accounts" search={{ companyId: company.id }}>
            <Button variant="outline" size="sm">Kontoplan</Button>
          </Link>
          {activeFy && (
            <Link to="/vouchers" search={{ companyId: company.id, fyId: activeFy.id }}>
              <Button variant="outline" size="sm">Verifikationer</Button>
            </Link>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function CreateCompanyForm({ onSuccess }: { onSuccess: () => void }) {
  const [name, setName] = useState('');
  const [orgNumber, setOrgNumber] = useState('');
  const [companyForm, setCompanyForm] = useState('AB');
  const [error, setError] = useState('');

  const mutation = useMutation({
    mutationFn: () =>
      companiesApi.create({
        name,
        org_number: orgNumber,
        company_form: companyForm,
      }),
    onSuccess: async (company) => {
      // Auto-create a fiscal year for the current calendar year
      const year = new Date().getFullYear();
      await fiscalYearsApi.create(company.id, {
        start_date: `${year}-01-01`,
        end_date: `${year}-12-31`,
      });
      onSuccess();
    },
    onError: (err: Error) => setError(err.message),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Skapa företag</CardTitle>
      </CardHeader>
      <CardContent>
        <form
          className="space-y-4"
          onSubmit={(e) => {
            e.preventDefault();
            setError('');
            mutation.mutate();
          }}
        >
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="name">Företagsnamn</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Mitt Företag AB"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="org">Organisationsnummer</Label>
              <Input
                id="org"
                value={orgNumber}
                onChange={(e) => setOrgNumber(e.target.value)}
                placeholder="5561234567"
                required
              />
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="form">Företagsform</Label>
            <select
              id="form"
              value={companyForm}
              onChange={(e) => setCompanyForm(e.target.value)}
              className="flex h-9 w-full max-w-[200px] rounded-md border border-input bg-transparent px-3 py-1 text-sm"
            >
              <option value="AB">Aktiebolag (AB)</option>
              <option value="EF">Enskild firma (EF)</option>
              <option value="HB">Handelsbolag (HB)</option>
              <option value="KB">Kommanditbolag (KB)</option>
              <option value="EK">Ekonomisk förening (EK)</option>
            </select>
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
          <Button type="submit" disabled={mutation.isPending}>
            {mutation.isPending ? 'Skapar...' : 'Skapa företag'}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
