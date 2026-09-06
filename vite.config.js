import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  base: '/bohemian-notesody/',

  plugins: [
    vue(),

    VitePWA({
      registerType: 'autoUpdate',

      manifest: {
        name: 'Bohemian Notesody',
        short_name: 'Notesody',
        description: 'Notizen, Aufgaben und Prozesse an einem Ort.',

        theme_color: '#006eab',
        background_color: '#f6f7f9',

        display: 'standalone',
        start_url: '/bohemian-notesody/',
        scope: '/bohemian-notesody/',

        icons: []
      }
    })
  ]
})