<script lang="ts">
  import type { AgentStatus } from "../lib/types";

  let { status = "idle", size = 14 }: { status?: AgentStatus; size?: number } = $props();
</script>

{#if status === "researching"}
  <!-- Eye: researching / learning process -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="status-icon researching"
    aria-label="Sedang riset"
  >
    <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z" />
    <circle cx="12" cy="12" r="3" />
  </svg>
{:else if status === "working"}
  <!-- Light bulb: working on solution -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="status-icon working"
    aria-label="Mengerjakan solusi"
  >
    <path d="M9 18h6" />
    <path d="M10 21h4" />
    <path d="M12 3a6 6 0 0 0-4 10.5c.8.7 1 1.5 1 2.5h6c0-1 .2-1.8 1-2.5A6 6 0 0 0 12 3Z" />
  </svg>
{:else if status === "delayed"}
  <!-- Clock: taking longer than expected -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="status-icon delayed"
    aria-label="Lebih lama dari perkiraan"
  >
    <circle cx="12" cy="12" r="10" />
    <polyline points="12 6 12 12 16 14" />
  </svg>
{:else if status === "done"}
  <!-- Heart: work completed -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="currentColor"
    class="status-icon done"
    aria-label="Selesai"
  >
    <path
      d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"
    />
  </svg>
{:else if status === "attention"}
  <!-- Attention: needs attention -->
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    class="status-icon attention"
    aria-label="Butuh perhatian"
  >
    <circle cx="12" cy="12" r="10" />
    <line x1="12" y1="8" x2="12" y2="12" />
    <line x1="12" y1="16" x2="12.01" y2="16" />
  </svg>
{/if}
<!-- idle: no icon -->

<style>
  .status-icon {
    flex: none;
    display: inline-block;
    vertical-align: middle;
  }
  .status-icon.researching {
    color: #4fa6f0;
  }
  .status-icon.working {
    color: #f5b301;
  }
  .status-icon.delayed {
    color: #ff8c42;
  }
  .status-icon.done {
    color: #e5484d;
  }
  .status-icon.attention {
    color: #e5484d;
  }
</style>
