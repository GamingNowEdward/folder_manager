<script setup lang="ts">
import AppShell from '@/app/AppShell.vue'
import { useGlobalInteractions } from '@/app/composables/useGlobalInteractions'

useGlobalInteractions()
</script>

<template>
  <AppShell />
</template>
