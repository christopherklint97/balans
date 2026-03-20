import { get, post, put, del } from './client';
import type { AuthResponse, UserInfo } from './types';
import type {
  Company,
  CreateCompany,
  FiscalYear,
  CreateFiscalYear,
  Account,
  Voucher,
  VoucherWithLines,
  CreateVoucher,
  TrialBalanceRow,
  SieImportPreview,
  SieImportResult,
  ValidationResult,
  ClosingParams,
  ClosingResult,
  ClosingStatus,
  IncomeStatement,
  BalanceSheet,
  AnnualReport,
  EligibilityResult,
  MultiYearOverview,
  AuditEntry,
  IxbrlPreview,
  FilingResult,
  Ink2Data,
  FixedAsset,
  CreateFixedAsset,
  DepreciationSummary,
} from './types';

// Auth
export const authApi = {
  register: (data: { email: string; password: string; name: string }) =>
    post<AuthResponse>('/auth/register', data),
  login: (data: { email: string; password: string }) =>
    post<AuthResponse>('/auth/login', data),
  me: () => get<UserInfo>('/auth/me'),
};

// Companies
export const companiesApi = {
  list: () => get<Company[]>('/companies'),
  get: (id: string) => get<Company>(`/companies/${id}`),
  create: (data: CreateCompany) => post<Company>('/companies', data),
  update: (id: string, data: Partial<Company>) => put<Company>(`/companies/${id}`, data),
};

// Fiscal Years
export const fiscalYearsApi = {
  list: (companyId: string) => get<FiscalYear[]>(`/companies/${companyId}/fiscal-years`),
  get: (id: string) => get<FiscalYear>(`/fiscal-years/${id}`),
  create: (companyId: string, data: CreateFiscalYear) =>
    post<FiscalYear>(`/companies/${companyId}/fiscal-years`, data),
};

// Accounts
export const accountsApi = {
  list: (companyId: string) => get<Account[]>(`/companies/${companyId}/accounts`),
};

// Vouchers
export const vouchersApi = {
  list: (fyId: string) => get<Voucher[]>(`/fiscal-years/${fyId}/vouchers`),
  get: (id: string) => get<VoucherWithLines>(`/vouchers/${id}`),
  create: (fyId: string, data: CreateVoucher) =>
    post<VoucherWithLines>(`/fiscal-years/${fyId}/vouchers`, data),
  delete: (id: string) => del<{ deleted: boolean }>(`/vouchers/${id}`),
};

// Reports
export const reportsApi = {
  trialBalance: (fyId: string) => get<TrialBalanceRow[]>(`/fiscal-years/${fyId}/trial-balance`),
};

function authHeader(): Record<string, string> {
  const token = localStorage.getItem('balans_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

// SIE Import/Export
export const sieApi = {
  preview: async (companyId: string, file: File): Promise<SieImportPreview> => {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch(`/api/companies/${companyId}/sie/preview`, {
      method: 'POST',
      headers: authHeader(),
      body: formData,
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(body.error || res.statusText);
    }
    return res.json();
  },

  import: async (companyId: string, file: File): Promise<SieImportResult> => {
    const formData = new FormData();
    formData.append('file', file);
    const res = await fetch(`/api/companies/${companyId}/sie/import`, {
      method: 'POST',
      headers: authHeader(),
      body: formData,
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(body.error || res.statusText);
    }
    return res.json();
  },

  exportUrl: (fyId: string, type: '1' | '4') =>
    `/api/fiscal-years/${fyId}/sie/export/${type}`,
};

// Closing (Årsbokslut)
export const closingApi = {
  validate: (fyId: string) => get<ValidationResult>(`/fiscal-years/${fyId}/closing/validate`),
  execute: (fyId: string, params: ClosingParams) =>
    post<ClosingResult>(`/fiscal-years/${fyId}/closing/execute`, params),
  status: (fyId: string) => get<ClosingStatus>(`/fiscal-years/${fyId}/closing/status`),
};

// Annual Report (Årsredovisning)
export const annualReportApi = {
  incomeStatement: (fyId: string) =>
    get<IncomeStatement>(`/fiscal-years/${fyId}/income-statement`),
  balanceSheet: (fyId: string) => get<BalanceSheet>(`/fiscal-years/${fyId}/balance-sheet`),
  full: (fyId: string) => get<AnnualReport>(`/fiscal-years/${fyId}/annual-report`),
  pdfUrl: (fyId: string) => `/api/fiscal-years/${fyId}/annual-report/pdf`,
};

// Compliance
export const complianceApi = {
  k2Eligibility: (companyId: string, fyId: string, employees?: number) =>
    get<EligibilityResult>(
      `/companies/${companyId}/fiscal-years/${fyId}/k2-eligibility${employees != null ? `?average_employees=${employees}` : ''}`,
    ),
  multiYear: (companyId: string) =>
    get<MultiYearOverview>(`/companies/${companyId}/multi-year`),
  auditLog: (companyId: string, limit?: number) =>
    get<AuditEntry[]>(`/companies/${companyId}/audit-log?limit=${limit || 100}`),
};

// Bolagsverket Filing
export const filingApi = {
  ixbrlPreview: (fyId: string) => get<IxbrlPreview>(`/fiscal-years/${fyId}/filing/ixbrl/preview`),
  ixbrlDownloadUrl: (fyId: string) => `/api/fiscal-years/${fyId}/filing/ixbrl`,
  submit: (fyId: string, production: boolean) =>
    post<FilingResult>(`/fiscal-years/${fyId}/filing/submit`, { production }),
};

// INK2 / Tax
export const taxApi = {
  ink2: (companyId: string, fyId: string) =>
    get<Ink2Data>(`/companies/${companyId}/fiscal-years/${fyId}/ink2`),
  sruDownloadUrl: (companyId: string, fyId: string) =>
    `/api/companies/${companyId}/fiscal-years/${fyId}/ink2/sru`,
};

// Fixed Assets
export const assetsApi = {
  list: (companyId: string) => get<FixedAsset[]>(`/companies/${companyId}/assets`),
  get: (id: string) => get<FixedAsset>(`/assets/${id}`),
  create: (companyId: string, data: CreateFixedAsset) =>
    post<FixedAsset>(`/companies/${companyId}/assets`, data),
  depreciation: (companyId: string, fyId: string) =>
    get<DepreciationSummary>(`/companies/${companyId}/fiscal-years/${fyId}/depreciation`),
  generateDepreciation: (companyId: string, fyId: string) =>
    post<{ vouchers_created: number; voucher_ids: string[] }>(
      `/companies/${companyId}/fiscal-years/${fyId}/depreciation/generate`,
      {},
    ),
};
