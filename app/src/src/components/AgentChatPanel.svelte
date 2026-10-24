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
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
