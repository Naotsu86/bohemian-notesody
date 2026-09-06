<script setup>
import { ref } from 'vue'

const activeView = ref('notes')

const compact = ref(false)
const compactMenuOpen = ref(false)

const quickNote = ref('')
const compactQuickNote = ref('')

const savedNotes = ref([])

const sections = {
  notes: {
    title: 'Notizen',
    description: 'Gedanken, Informationen und Dokumentationen festhalten.'
  },

  tasks: {
    title: 'Aufgaben',
    description: 'Eigene und gemeinsame pendente Aktivitäten verwalten.'
  },

  processes: {
    title: 'Prozesse',
    description: 'Abläufe grafisch darstellen und dokumentieren.'
  }
}

function createNote(text) {
  const cleanText = text.trim()

  if (!cleanText) return false

  savedNotes.value.unshift({
    id: Date.now(),
    text: cleanText,
    createdAt: new Date().toLocaleString('de-DE')
  })

  return true
}

function saveQuickNote() {
  if (createNote(quickNote.value)) {
    quickNote.value = ''
  }
}

function saveCompactQuickNote() {
  if (createNote(compactQuickNote.value)) {
    compactQuickNote.value = ''
  }
}

function openCompactMode() {
  compact.value = true
  compactMenuOpen.value = false
}

function leaveCompactMode() {
  compact.value = false
  compactMenuOpen.value = false
}

function openSection(section) {
  activeView.value = section
  compact.value = false
  compactMenuOpen.value = false
}

function newTask() {
  openSection('tasks')
}

function newProcess() {
  openSection('processes')
}
</script>

<template>
  <div class="app-shell" :class="{ compact }">

    <!-- ================================================= -->
    <!-- KOMPAKTMODUS -->
    <!-- ================================================= -->

    <div
      v-if="compact"
      class="compact-workspace"
    >
      <div class="compact-wrapper">

        <!-- kleines Notizbuch -->
        <button
          class="notebook-button"
          title="Notesody Schnellmenü"
          @click="compactMenuOpen = !compactMenuOpen"
        >
          <span class="book-spine"></span>

          <span class="book-lines">
            <span></span>
            <span></span>
            <span></span>
          </span>
        </button>


        <!-- kleines Popup -->
        <div
          v-if="compactMenuOpen"
          class="compact-menu"
        >
          <div class="compact-menu-header">

            <div class="compact-brand-icon">
              <span></span>
              <span></span>
              <span></span>
            </div>

            <div>
              <strong>Bohemian Notesody</strong>
              <span>Schnellzugriff</span>
            </div>

          </div>


          <div class="compact-section">

            <div class="compact-section-title">
              Schnellnotiz
            </div>

            <textarea
              v-model="compactQuickNote"
              placeholder="Was möchtest du festhalten?"
              @keydown.ctrl.enter.prevent="saveCompactQuickNote"
            ></textarea>

            <div class="compact-save-row">

              <span>
                Strg + Enter
              </span>

              <button
                class="primary-button compact-save-button"
                @click="saveCompactQuickNote"
              >
                Speichern
              </button>

            </div>

          </div>


          <div class="compact-actions">

            <button
              class="compact-action"
              @click="newTask"
            >
              <span class="compact-action-icon task">
                ✓
              </span>

              <div>
                <strong>Neue Aufgabe</strong>
                <span>Pendente Aktivität anlegen</span>
              </div>

              <span class="compact-arrow">
                ›
              </span>
            </button>


            <button
              class="compact-action"
              @click="newProcess"
            >
              <span class="compact-action-icon process">
                ◇
              </span>

              <div>
                <strong>Neuer Prozess</strong>
                <span>Ablauf grafisch erstellen</span>
              </div>

              <span class="compact-arrow">
                ›
              </span>
            </button>

          </div>


          <button
            class="open-full-app"
            @click="leaveCompactMode"
          >
            <span>↗</span>
            Vollständig öffnen
          </button>

        </div>

      </div>
    </div>


    <!-- ================================================= -->
    <!-- NORMALE APP -->
    <!-- ================================================= -->

    <template v-else>

      <header class="topbar">

        <div class="brand">

          <div class="brand-icon">
            <span class="brand-spine"></span>
            <span class="brand-line"></span>
            <span class="brand-line"></span>
          </div>

          <div>
            <div class="brand-name">
              Bohemian Notesody
            </div>

            <div class="brand-subtitle">
              Notizen · Aufgaben · Prozesse
            </div>
          </div>

        </div>


        <div class="topbar-actions">

          <button
            class="icon-button"
            title="Kompaktmodus"
            @click="openCompactMode"
          >
            ▭
          </button>

          <button
            class="user-button"
            title="Benutzer"
          >
            AB
          </button>

        </div>

      </header>


      <div class="workspace">

        <!-- ================================================= -->
        <!-- NAVIGATION -->
        <!-- ================================================= -->

        <aside class="sidebar">

          <nav class="main-nav">

            <button
              class="nav-item"
              :class="{ active: activeView === 'notes' }"
              @click="activeView = 'notes'"
            >
              <span class="nav-icon">▤</span>
              <span>Notizen</span>
            </button>


            <button
              class="nav-item"
              :class="{ active: activeView === 'tasks' }"
              @click="activeView = 'tasks'"
            >
              <span class="nav-icon">✓</span>
              <span>Aufgaben</span>
            </button>


            <button
              class="nav-item"
              :class="{ active: activeView === 'processes' }"
              @click="activeView = 'processes'"
            >
              <span class="nav-icon">◇</span>
              <span>Prozesse</span>
            </button>

          </nav>


          <div class="sidebar-bottom">

            <button class="nav-item muted">
              <span class="nav-icon">♙</span>
              <span>Gruppen</span>
            </button>

            <button class="nav-item muted">
              <span class="nav-icon">⚙</span>
              <span>Einstellungen</span>
            </button>

          </div>

        </aside>


        <!-- ================================================= -->
        <!-- HAUPTBEREICH -->
        <!-- ================================================= -->

        <main class="main-content">

          <div class="page-heading">

            <div>
              <h1>
                {{ sections[activeView].title }}
              </h1>

              <p>
                {{ sections[activeView].description }}
              </p>
            </div>

            <button class="primary-button">
              + Neu
            </button>

          </div>


          <!-- ================================================= -->
          <!-- NOTIZEN -->
          <!-- ================================================= -->

          <template v-if="activeView === 'notes'">

            <section class="quick-note-card">

              <div class="card-heading">

                <div>
                  <h2>Schnellnotiz</h2>

                  <p>
                    Einfach losschreiben und später einsortieren.
                  </p>
                </div>

                <span class="small-badge">
                  QUICK
                </span>

              </div>


              <textarea
                v-model="quickNote"
                placeholder="Was möchtest du festhalten?"
                @keydown.ctrl.enter.prevent="saveQuickNote"
              ></textarea>


              <div class="quick-note-footer">

                <span class="shortcut-hint">
                  Strg + Enter zum Speichern
                </span>

                <button
                  class="primary-button"
                  @click="saveQuickNote"
                >
                  Speichern
                </button>

              </div>

            </section>


            <section class="section-block">

              <div class="section-title-row">
                <h2>Zuletzt verwendet</h2>
              </div>


              <div class="dashboard-grid">

                <button
                  class="dashboard-card notes-card"
                  @click="openSection('notes')"
                >

                  <div class="dashboard-icon">
                    ▤
                  </div>

                  <div>
                    <strong>Notizen</strong>
                    <span>Dokumentieren und festhalten</span>
                  </div>

                </button>


                <button
                  class="dashboard-card tasks-card"
                  @click="openSection('tasks')"
                >

                  <div class="dashboard-icon">
                    ✓
                  </div>

                  <div>
                    <strong>Aufgaben</strong>
                    <span>Pendente Aktivitäten verwalten</span>
                  </div>

                </button>


                <button
                  class="dashboard-card process-card"
                  @click="openSection('processes')"
                >

                  <div class="dashboard-icon">
                    ◇
                  </div>

                  <div>
                    <strong>Prozesse</strong>
                    <span>Abläufe grafisch darstellen</span>
                  </div>

                </button>

              </div>

            </section>


            <section class="section-block">

              <div class="section-title-row">

                <h2>
                  Meine Schnellnotizen
                </h2>

                <span class="count">
                  {{ savedNotes.length }}
                </span>

              </div>


              <div
                v-if="savedNotes.length === 0"
                class="empty-state"
              >

                <div class="empty-icon">
                  ▤
                </div>

                <strong>
                  Noch keine Notizen
                </strong>

                <span>
                  Deine Schnellnotizen erscheinen hier.
                </span>

              </div>


              <div
                v-else
                class="note-list"
              >

                <article
                  v-for="note in savedNotes"
                  :key="note.id"
                  class="note-card"
                >

                  <div class="note-marker"></div>

                  <div class="note-body">

                    <p>
                      {{ note.text }}
                    </p>

                    <span>
                      {{ note.createdAt }}
                    </span>

                  </div>

                </article>

              </div>

            </section>

          </template>


          <!-- ================================================= -->
          <!-- AUFGABEN -->
          <!-- ================================================= -->

          <template v-if="activeView === 'tasks'">

            <section class="placeholder-card">

              <div class="placeholder-icon">
                ✓
              </div>

              <h2>
                Aufgaben & pendente Aktivitäten
              </h2>

              <p>
                Hier entstehen später persönliche Aufgaben,
                Gruppenzuweisungen, Ergebnisse und Statusmeldungen.
              </p>


              <div class="preview-task">

                <span class="task-check"></span>

                <div>
                  <strong>
                    Beispielaufgabe
                  </strong>

                  <span>
                    Offen · Dir zugewiesen
                  </span>
                </div>

                <span class="status-badge">
                  OFFEN
                </span>

              </div>

            </section>

          </template>


          <!-- ================================================= -->
          <!-- PROZESSE -->
          <!-- ================================================= -->

          <template v-if="activeView === 'processes'">

            <section class="placeholder-card process-preview">

              <div class="placeholder-icon">
                ◇
              </div>

              <h2>
                Prozesseditor
              </h2>

              <p>
                Hier bauen wir später deinen kleinen Grafik-Köcher
                für Prozesse ein.
              </p>


              <div class="process-canvas-preview">

                <div class="process-node start-node">
                  Start
                </div>

                <div class="process-arrow">
                  ↓
                </div>

                <div class="process-node action-node">
                  Vorgang bearbeiten
                </div>

                <div class="process-arrow">
                  ↓
                </div>

                <div class="decision-node">
                  <span>OK?</span>
                </div>

              </div>

            </section>

          </template>

        </main>

      </div>

    </template>

  </div>
</template>