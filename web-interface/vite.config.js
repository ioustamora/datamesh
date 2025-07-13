import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src')
    }
  },
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        secure: false
      },
      '/ws': {
        target: 'ws://localhost:8080',
        ws: true,
        changeOrigin: true
      }
    }
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: process.env.NODE_ENV === 'development',
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: process.env.NODE_ENV === 'production',
        drop_debugger: process.env.NODE_ENV === 'production'
      }
    },
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html')
      },
      output: {
        manualChunks: {
          // Vendor chunks
          'vendor-vue': ['vue', 'vue-router', 'pinia'],
          'vendor-ui': ['element-plus', '@element-plus/icons-vue'],
          'vendor-utils': ['axios', 'dayjs', 'file-saver', 'mime-types'],
          'vendor-charts': ['chart.js', 'vue-chartjs'],
          'vendor-i18n': ['vue-i18n'],
          'vendor-workbox': [
            'workbox-precaching',
            'workbox-routing',
            'workbox-strategies',
            'workbox-cacheable-response',
            'workbox-expiration',
            'workbox-background-sync',
            'workbox-broadcast-update'
          ],
          
          // Feature chunks
          'feature-dashboard': [
            './src/views/Dashboard.vue',
            './src/components/dashboard/DashboardCard.vue'
          ],
          'feature-files': [
            './src/views/FileManager.vue',
            './src/components/files/FileList.vue',
            './src/components/files/FileUpload.vue'
          ],
          'feature-governance': [
            './src/views/Governance.vue',
            './src/components/governance/ProposalCard.vue'
          ],
          'feature-analytics': [
            './src/views/Analytics.vue',
            './src/components/charts/PerformanceChart.vue',
            './src/components/charts/UsageChart.vue'
          ]
        },
        chunkFileNames: (chunkInfo) => {
          const facadeModuleId = chunkInfo.facadeModuleId
          if (facadeModuleId) {
            if (facadeModuleId.includes('node_modules')) {
              return 'vendor/[name].[hash].js'
            } else if (facadeModuleId.includes('views/')) {
              return 'pages/[name].[hash].js'
            } else if (facadeModuleId.includes('components/')) {
              return 'components/[name].[hash].js'
            }
          }
          return 'chunks/[name].[hash].js'
        },
        assetFileNames: (assetInfo) => {
          const info = assetInfo.name.split('.')
          const extType = info[info.length - 1]
          if (/png|jpe?g|svg|gif|tiff|bmp|ico/i.test(extType)) {
            return 'images/[name].[hash].[ext]'
          } else if (/woff|woff2|eot|ttf|otf/i.test(extType)) {
            return 'fonts/[name].[hash].[ext]'
          } else if (/css/i.test(extType)) {
            return 'styles/[name].[hash].[ext]'
          }
          return 'assets/[name].[hash].[ext]'
        }
      }
    },
    reportCompressedSize: false,
    chunkSizeWarningLimit: 1000
  },
  optimizeDeps: {
    include: [
      'vue',
      'vue-router',
      'pinia',
      'element-plus',
      '@element-plus/icons-vue',
      'axios',
      'dayjs',
      'chart.js',
      'vue-chartjs',
      'vue-i18n'
    ]
  },
  define: {
    __VUE_I18N_FULL_INSTALL__: true,
    __VUE_I18N_LEGACY_API__: false,
    __INTLIFY_PROD_DEVTOOLS__: false
  }
})