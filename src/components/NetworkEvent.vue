<script setup lang="ts">
  const networkStore = useNetworkStore()

  const { currentNetwork } = storeToRefs(networkStore)

  const events = computed(() => {
    // @ts-expect-error record
    console.log(currentNetwork.value?.detail.events)
    return (currentNetwork.value?.detail?.events??[]).map((event) => {
      console.log(event)
      return {
        time: new Date(event.time).toLocaleString(),
        content: JSON.stringify(event.event, null, '\t'),
      }
    }) || []
  })
</script>

<template>
  <n-timeline v-if="currentNetwork?.detail">
    <n-timeline-item v-for="(event, index) in events" :key="event.time" type="info" :time="event.time"
      :line-type="index === events.length - 1 ? 'dashed' : undefined">
      <n-code :code="event.content" language="json" word-wrap />
    </n-timeline-item>
    <n-timeline-item />
  </n-timeline>
</template>
