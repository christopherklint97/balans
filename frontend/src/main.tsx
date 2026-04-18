import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { RouterProvider, createRouter } from '@tanstack/react-router';
import {
  QueryClient,
  QueryClientProvider,
  QueryCache,
  MutationCache,
} from '@tanstack/react-query';
import { toast } from 'sonner';
import { AuthProvider } from './auth/context';
import { ThemeProvider } from './hooks/use-theme';
import { FiscalYearProvider } from './hooks/use-fiscal-year';
import { Toaster } from './components/ui/sonner';
import { ApiError } from './api/client';
import { routeTree } from './routeTree.gen';
import './index.css';

function messageFromError(err: unknown): string {
  if (err instanceof ApiError) return err.message;
  if (err instanceof Error) return err.message;
  return 'Ett oväntat fel uppstod';
}

function shouldToast(err: unknown): boolean {
  if (err instanceof ApiError && err.status === 401) return false;
  return true;
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      retry: 1,
    },
  },
  queryCache: new QueryCache({
    onError: (err) => {
      if (shouldToast(err)) toast.error(messageFromError(err));
    },
  }),
  mutationCache: new MutationCache({
    onError: (err) => {
      if (shouldToast(err)) toast.error(messageFromError(err));
    },
  }),
});

const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <AuthProvider>
          <FiscalYearProvider>
            <RouterProvider router={router} />
            <Toaster />
          </FiscalYearProvider>
        </AuthProvider>
      </QueryClientProvider>
    </ThemeProvider>
  </StrictMode>,
);
