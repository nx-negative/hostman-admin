import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { createRouter, RouterProvider } from '@tanstack/react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { routeTree } from './routeTree.gen'
import { AuthProvider, useAuth } from '@/app/auth'
import { ThemeProvider } from '@/app/theme'
import { Toaster } from '@/components/ui/toast'
import './index.css'

const router = createRouter({ routeTree })

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: 1, staleTime: 10_000 } },
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

function Inner() {
  const auth = useAuth()
  return <RouterProvider router={router} context={{ auth }} />
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <Toaster />
        <AuthProvider>
          <Inner />
        </AuthProvider>
      </ThemeProvider>
    </QueryClientProvider>
  </StrictMode>,
)
