/// <reference types="vitest/config" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { TanStackRouterVite } from '@tanstack/router-plugin/vite'

export default defineConfig({
  plugins: [TanStackRouterVite(), react(), tailwindcss()],
  resolve: { alias: { '@': import.meta.dirname + '/src' } },
  server: { proxy: { '/api': { target: 'http://127.0.0.1:8080', changeOrigin: true } } },
  test: { environment: 'node', include: ['src/**/*.test.{ts,tsx}'] },
})
