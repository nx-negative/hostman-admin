/// <reference types="vitest/config" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import { tanstackRouter } from '@tanstack/router-plugin/vite'

export default defineConfig({
  plugins: [tanstackRouter(), react(), tailwindcss()],
  resolve: { alias: { '@': import.meta.dirname + '/src' } },
  server: { proxy: { '/api': { target: 'http://127.0.0.1:8080', changeOrigin: true } } },
  test: { environment: 'node', include: ['src/**/*.test.{ts,tsx}'] },
    build: {
    // keep the main bundle under the 500 kB report threshold by code-splitting
    // heavy vendors into their own async/shared chunks (qrcode is loaded lazily
    // on demand in the login wizard).
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('react') || id.includes('react-dom')) return 'react'
          if (id.includes('framer-motion')) return 'motion'
        },
      },
    },
  },
})
