<script lang="ts">
  import type { ModuleInfo } from "$lib/api";

  let {
    moduleCount = null,
    modules = [],
    minTotalValue = 12,
  }: {
    moduleCount: number | null;
    modules: ModuleInfo[];
    minTotalValue: number;
  } = $props();

  const filteredModuleCount = $derived(
    modules.filter(
      (module) =>
        module.parts.reduce((total, part) => total + part.value, 0) >= minTotalValue
    ).length
  );
</script>

<div class="rounded-lg border border-border/60 bg-card/40 p-4 space-y-1">
  <div class="text-base font-semibold text-foreground">数据状态</div>
  <div class="text-sm text-muted-foreground">
    模组数量：{moduleCount ?? "未同步"}
  </div>
  <div class="text-sm text-muted-foreground">
    总值筛选后：{moduleCount === null ? "未同步" : filteredModuleCount}
  </div>
</div>

