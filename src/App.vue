<template>
  <div 
    class="container" 
    :style="containerStyle" 
    @mousedown="focusContainer"
    tabindex="0" 
    ref="containerRef"
  >
    <div class="header-row" :style="{ height: headerHeight + 'px' }">
      <div class="corner-cell" :style="{ width: timeColWidth + 'px' }"></div>
      <div v-for="(day, index) in headers" :key="index" class="header-cell">
        {{ day }}
      </div>
    </div>

    <div class="grid-body">
      <div class="time-col" :style="{ width: timeColWidth + 'px' }">
        <div v-for="(time, index) in timeSlots" :key="index" class="time-cell">
          {{ time }}
        </div>
      </div>

      <div class="courses-grid" :style="{ gridTemplateColumns: `repeat(${headers.length}, 1fr)` }">
        <div v-for="(colData, colIndex) in processedGrid" :key="colIndex" class="day-column">
          <div 
            v-for="cell in colData" 
            :key="cell.rowStart"
            :style="getCellStyle(cell, colIndex)"
            class="course-card"
            :class="{ 'focused': isFocused(cell, colIndex), 'empty-card': !cell.name }"
            @click="setFocus(cell.rowStart, colIndex)"
            @dblclick="startEdit"
          >
            <div v-if="!(isEditing && isFocused(cell, colIndex))" class="card-content">
              {{ cell.name }}
            </div>
            <input 
              v-else
              ref="editInput"
              v-model="editValue"
              @blur="saveEdit"
              @keydown.enter.stop="saveEdit"
              @keydown.esc.stop="isEditing = false"
              class="edit-input"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, computed, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

const headers = ref([]);
const timeSlots = ref([]);
const rawGrid = ref([]);
const theme = ref({
  background: '#191724',
  foreground: '#e0def4',
  palette: [],
  selection_background: '#403d52',
  font_size: 14
});

const cursor = ref({ row: 0, col: 0 });
const containerRef = ref(null);
const isEditing = ref(false);
const editValue = ref("");
const editInput = ref(null);

const winWidth = ref(window.innerWidth);
const winHeight = ref(window.innerHeight);
const timeColWidth = 120;
const headerHeight = 50;

const adaptiveFontSize = computed(() => {
  if (!timeSlots.value.length || !headers.value.length) return theme.value.font_size;
  const availableH = winHeight.value - headerHeight;
  const availableW = winWidth.value - timeColWidth;
  const cellH = availableH / timeSlots.value.length;
  const cellW = availableW / headers.value.length;
  const size = Math.min(cellH * 0.22, cellW * 0.16, theme.value.font_size);
  return Math.max(size, 9);
});

const containerStyle = computed(() => ({
  '--bg-color': theme.value.background,
  '--fg-color': theme.value.foreground,
  '--selection-bg': theme.value.selection_background,
  'font-size': adaptiveFontSize.value + 'px'
}));

const processedGrid = computed(() => {
  if (!headers.value.length || !rawGrid.value.length) return [];
  const result = [];
  for (let c = 0; c < headers.value.length; c++) {
    const colCells = [];
    let r = 0;
    while (r < timeSlots.value.length) {
      const currentName = rawGrid.value[r]?.[c] || "";
      let span = 1;
      if (currentName !== "") {
        while (r + span < timeSlots.value.length && rawGrid.value[r+span]?.[c] === currentName) {
          span++;
        }
      }
      colCells.push({ name: currentName, rowStart: r, span });
      r += span;
    }
    result.push(colCells);
  }
  return result;
});

function updateWinSize() {
  winWidth.value = window.innerWidth;
  winHeight.value = window.innerHeight;
}

async function loadAll() {
  theme.value = await invoke('get_theme');
  const data = await invoke('get_schedule');
  headers.value = data.headers;
  timeSlots.value = data.time_slots;
  rawGrid.value = data.grid;
}

const handleGlobalKeydown = (e) => {
  if (isEditing.value) return;
  const maxR = timeSlots.value.length - 1;
  const maxC = headers.value.length - 1;
  if (e.key === 'ArrowUp') cursor.value.row = Math.max(0, cursor.value.row - 1);
  else if (e.key === 'ArrowDown') cursor.value.row = Math.min(maxR, cursor.value.row + 1);
  else if (e.key === 'ArrowLeft') cursor.value.col = Math.max(0, cursor.value.col - 1);
  else if (e.key === 'ArrowRight') cursor.value.col = Math.min(maxC, cursor.value.col + 1);
  else if (e.key === 'Enter') { e.preventDefault(); startEdit(); }
};

onMounted(() => {
  loadAll();
  window.addEventListener('keydown', handleGlobalKeydown);
  window.addEventListener('resize', updateWinSize);
  focusContainer();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown);
  window.removeEventListener('resize', updateWinSize);
});

function focusContainer() { nextTick(() => containerRef.value?.focus()); }

function getCardColor(name) {
  if (!name || !name.trim()) return 'transparent';
  let h = 0;
  for (let i = 0; i < name.length; i++) {
    h = name.charCodeAt(i) + ((h << 5) - h);
  }
  const index = Math.abs(h) % theme.value.palette.length;
  return theme.value.palette[index];
}

function getCellStyle(cell, colIndex) {
  return {
    height: `calc(${cell.span * (100 / timeSlots.value.length)}% - 6px)`,
    top: `calc(${cell.rowStart * (100 / timeSlots.value.length)}% + 3px)`,
    backgroundColor: cell.name ? getCardColor(cell.name) : 'transparent',
    opacity: cell.name ? 0.8 : 1,
    color: cell.name ? theme.value.background : theme.value.foreground,
    fontSize: (cell.name.length > 8 ? adaptiveFontSize.value * 0.9 : adaptiveFontSize.value) + 'px'
  };
}

function isFocused(cell, colIndex) {
  return cursor.value.col === colIndex && 
         cursor.value.row >= cell.rowStart && 
         cursor.value.row < (cell.rowStart + cell.span);
}

function setFocus(r, c) { if (!isEditing.value) cursor.value = { row: r, col: c }; }

function startEdit() {
  isEditing.value = true;
  editValue.value = rawGrid.value[cursor.value.row][cursor.value.col];
  nextTick(() => editInput.value?.[0]?.focus());
}

async function saveEdit() {
  if (!isEditing.value) return;
  const { row, col } = cursor.value;
  rawGrid.value[row][col] = editValue.value;
  isEditing.value = false;
  await invoke('save_cell', { row, col, value: editValue.value });
  focusContainer();
}
</script>

<style scoped>
.container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  background: var(--bg-color);
  outline: none;
  overflow: hidden;
  color: var(--fg-color);
  user-select: none;
  padding-left: 10px;
  box-sizing: border-box;
}
.header-row {
  display: flex;
  flex-shrink: 0;
  border-bottom: 1px solid rgba(255,255,255,0.08);
}
.corner-cell { flex-shrink: 0; }
.header-cell {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  opacity: 0.85;
  letter-spacing: 0.5px;
}
.grid-body { flex: 1; display: flex; overflow: hidden; }
.time-col {
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  border-right: 1px solid rgba(255,255,255,0.08);
}
.time-cell {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.5;
  font-size: 0.85em;
  font-weight: 400;
}
.courses-grid { flex: 1; display: grid; height: 100%; }
.day-column { position: relative; height: 100%; border-right: 1px solid rgba(255,255,255,0.04); }
.course-card {
  position: absolute;
  left: 3px;
  right: 3px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 6px;
  box-sizing: border-box;
  font-weight: 600;
  border: 2px solid transparent;
  cursor: pointer;
  transition: border-color 0.15s ease;
}
.course-card.focused {
  border-color: var(--fg-color) !important;
  z-index: 10;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
.card-content {
  text-align: center;
  word-break: break-all;
  line-height: 1.25;
}
.edit-input {
  width: 100%;
  height: 100%;
  background: rgba(255, 255, 255, 0.95);
  color: #1a1b26;
  border: none;
  outline: none;
  text-align: center;
  border-radius: 6px;
  font-family: inherit;
  font-weight: 700;
}
</style>/style>/style>