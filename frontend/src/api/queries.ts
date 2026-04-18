import { get, post, put, del } from './client';
import type { AuthResponse, UserInfo, AdminUser, CompanyUser, AppConfig } from './types';
import type {
  Company,
  CreateCompany,
  FiscalYear,
  CreateFiscalYear,
  Account,
  Voucher,
  VoucherWithLines,
  CreateVoucher,
  AttachmentMeta,
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
  DirectorsReportTexts,
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

// Attachments
export const attachmentsApi = {
  list: (voucherId: string) => get<AttachmentMeta[]>(`/vouchers/${voucherId}/attachments`),
  downloadUrl: (voucherId: string, attachmentId: string) =>
    `/api/vouchers/${voucherId}/attachments/${attachmentId}`,
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
  downloadPdf: async (fyId: string) => {
    const token = localStorage.getItem('balans_token');
    const res = await fetch(`/api/fiscal-years/${fyId}/annual-report/pdf`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });
    if (!res.ok) {
      const body = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(body.error || 'Kunde inte ladda ner PDF');
    }
    const blob = await res.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'arsredovisning.pdf';
    a.rel = 'noopener';
    a.target = '_blank';
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  },
  getTexts: (fyId: string) =>
    get<DirectorsReportTexts>(`/fiscal-years/${fyId}/directors-report-texts`),
  updateTexts: (fyId: string, data: Partial<DirectorsReportTexts>) =>
    put<DirectorsReportTexts>(`/fiscal-years/${fyId}/directors-report-texts`, data),
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

// Admin
export const adminApi = {
  listUsers: () => get<AdminUser[]>('/admin/users'),
  listPendingUsers: () => get<AdminUser[]>('/admin/users/pending'),
  approveUser: (id: string) => put<AdminUser>(`/admin/users/${id}/approve`, {}),
  rejectUser: (id: string) => put<AdminUser>(`/admin/users/${id}/reject`, {}),
  activateUser: (id: string) => put<AdminUser>(`/admin/users/${id}/activate`, {}),
  deactivateUser: (id: string) => put<AdminUser>(`/admin/users/${id}/deactivate`, {}),
  changeUserRole: (id: string, role: string) =>
    put<AdminUser>(`/admin/users/${id}/role`, { role }),
  listCompanyUsers: (companyId: string) =>
    get<CompanyUser[]>(`/admin/companies/${companyId}/users`),
  addCompanyUser: (companyId: string, userId: string, role: string) =>
    post<{ ok: boolean }>(`/admin/companies/${companyId}/users`, { user_id: userId, role }),
  changeCompanyRole: (companyId: string, userId: string, role: string) =>
    put<{ ok: boolean }>(`/admin/companies/${companyId}/users/${userId}`, { role }),
  removeCompanyUser: (companyId: string, userId: string) =>
    del<{ ok: boolean }>(`/admin/companies/${companyId}/users/${userId}`),
  getConfig: () => get<AppConfig>('/admin/config'),
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
