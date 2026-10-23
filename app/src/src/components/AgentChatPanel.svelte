<script lang="ts">
  import ChatPanel from "./ChatPanel.svelte";
  import type { Agent } from "../lib/types";

  interface Props {
    chatAgent: Agent;
  }
  let { chatAgent }: Props = $props();
</script>

<!-- Middle column: chat area (widest) -->
<div class="chat-col">
  <ChatPanel agent={chatAgent} header={false} />
</div>

<style>
  .chat-col {
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
</style>
