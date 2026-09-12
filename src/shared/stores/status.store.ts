import { ref } from 'vue'
import { defineStore } from 'pinia'
import { STATUS_READY, STATUS_RESET_MS } from '@/shared/constants'

export const useStatusStore = defineStore('status', () => {
  const message = ref(STATUS_READY)
  let resetTimer: ReturnType<typeof setTimeout> | null = null

  function setStatus(text: string, timeout = STATUS_RESET_MS): void {
    if (resetTimer) clearTimeout(resetTimer)
    message.value = text
    if (timeout > 0) {
      resetTimer = setTimeout(() => {
        message.value = STATUS_READY
      }, timeout)
    }
  }

  return { message, setStatus }
})
