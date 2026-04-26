<script lang="ts">
  import {
    HEADER_LAYOUT_COMPONENT_LABELS,
    HEADER_LAYOUT_ZONE_IDS,
    HEADER_LAYOUT_ZONE_LABELS,
    cloneHeaderCustomLayout,
    normalizeHeaderLayout,
    type HeaderCustomLayout,
    type HeaderLayoutComponentId,
    type HeaderLayoutZone,
  } from "$lib/live-header-layout";
  import SettingsSlider from "./settings-slider.svelte";

  let {
    layout = $bindable(cloneHeaderCustomLayout()),
  }: {
    layout: HeaderCustomLayout;
  } = $props();

  const normalizedLayout = $derived(normalizeHeaderLayout(layout));

  function setLayout(nextLayout: HeaderCustomLayout) {
    layout = normalizeHeaderLayout(nextLayout);
  }

  function moveRow(rowIndex: number, direction: -1 | 1) {
    const targetIndex = rowIndex + direction;
    const nextLayout = cloneHeaderCustomLayout(normalizedLayout);
    if (targetIndex < 0 || targetIndex >= nextLayout.rows.length) return;

    const currentRow = nextLayout.rows[rowIndex];
    const targetRow = nextLayout.rows[targetIndex];
    if (!currentRow || !targetRow) return;

    nextLayout.rows[rowIndex] = targetRow;
    nextLayout.rows[targetIndex] = currentRow;
    setLayout(nextLayout);
  }

  function removeComponentFromZones(
    row: HeaderCustomLayout["rows"][number],
    componentId: HeaderLayoutComponentId,
  ) {
    for (const zoneId of HEADER_LAYOUT_ZONE_IDS) {
      row.zones[zoneId] = row.zones[zoneId].filter(
        (nextComponentId) => nextComponentId !== componentId,
      );
    }
  }

  function moveComponentInZone(
    rowIndex: number,
    zoneId: HeaderLayoutZone,
    componentId: HeaderLayoutComponentId,
    direction: -1 | 1,
  ) {
    const nextLayout = cloneHeaderCustomLayout(normalizedLayout);
    const row = nextLayout.rows[rowIndex];
    if (!row) return;

    const zoneComponents = row.zones[zoneId];
    const componentIndex = zoneComponents.indexOf(componentId);
    const targetIndex = componentIndex + direction;
    if (
      componentIndex < 0 ||
      targetIndex < 0 ||
      targetIndex >= zoneComponents.length
    ) {
      return;
    }

    const targetComponent = zoneComponents[targetIndex];
    if (!targetComponent) return;

    zoneComponents[targetIndex] = componentId;
    zoneComponents[componentIndex] = targetComponent;
    setLayout(nextLayout);
  }

  function moveComponentToZone(
    rowIndex: number,
    currentZoneId: HeaderLayoutZone,
    componentId: HeaderLayoutComponentId,
    targetZoneId: HeaderLayoutZone,
  ) {
    if (currentZoneId === targetZoneId) return;

    const nextLayout = cloneHeaderCustomLayout(normalizedLayout);
    const row = nextLayout.rows[rowIndex];
    if (!row) return;

    removeComponentFromZones(row, componentId);
    row.zones[targetZoneId].push(componentId);
    setLayout(nextLayout);
  }

  function moveComponentToRow(
    rowIndex: number,
    zoneId: HeaderLayoutZone,
    componentId: HeaderLayoutComponentId,
    targetRowIndex: number,
  ) {
    const nextLayout = cloneHeaderCustomLayout(normalizedLayout);
    const sourceRow = nextLayout.rows[rowIndex];
    const targetRow = nextLayout.rows[targetRowIndex];
    if (!sourceRow || !targetRow) return;

    removeComponentFromZones(sourceRow, componentId);
    targetRow.zones[zoneId].push(componentId);
    setLayout(nextLayout);
  }

  function moveComponentToNewRow(
    rowIndex: number,
    zoneId: HeaderLayoutZone,
    componentId: HeaderLayoutComponentId,
  ) {
    const nextLayout = cloneHeaderCustomLayout(normalizedLayout);
    const sourceRow = nextLayout.rows[rowIndex];
    if (!sourceRow) return;

    removeComponentFromZones(sourceRow, componentId);
    nextLayout.rows.splice(rowIndex + 1, 0, {
      id: `custom-${Date.now()}`,
      zones: {
        start: zoneId === "start" ? [componentId] : [],
        end: zoneId === "end" ? [componentId] : [],
      },
    });
    setLayout(nextLayout);
  }
</script>

<div class="space-y-4">
  <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
    <SettingsSlider
      bind:value={layout.rowGap}
      min={0}
      max={24}
      step={1}
      label="行间距"
      description="标题栏多行之间的距离"
      unit="px"
    />
    <SettingsSlider
      bind:value={layout.itemGap}
      min={0}
      max={24}
      step={1}
      label="组件间距"
      description="同一行内组件之间的距离"
      unit="px"
    />
  </div>

  <div class="space-y-3">
    {#each normalizedLayout.rows as row, rowIndex (row.id + rowIndex)}
      <div class="rounded-lg border border-border/60 bg-card/30 p-3 space-y-3">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div>
            <div class="text-sm font-semibold text-foreground">
              第 {rowIndex + 1} 行
            </div>
            <div class="text-xs text-muted-foreground">
              调整本行组件所在区域、顺序和换行位置。
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              type="button"
              class="rounded-md border border-border/60 px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-popover/60 hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
              disabled={rowIndex === 0}
              onclick={() => moveRow(rowIndex, -1)}
            >
              行上移
            </button>
            <button
              type="button"
              class="rounded-md border border-border/60 px-2 py-1 text-xs text-muted-foreground transition-colors hover:bg-popover/60 hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
              disabled={rowIndex === normalizedLayout.rows.length - 1}
              onclick={() => moveRow(rowIndex, 1)}
            >
              行下移
            </button>
          </div>
        </div>

        <div class="grid grid-cols-1 gap-3 xl:grid-cols-2">
          {#each HEADER_LAYOUT_ZONE_IDS as zoneId (zoneId)}
            <div
              class="rounded-md border border-border/50 bg-background/40 p-2"
            >
              <div class="mb-2 text-xs font-semibold text-muted-foreground">
                {HEADER_LAYOUT_ZONE_LABELS[zoneId]}
              </div>
              {#if row.zones[zoneId].length === 0}
                <div
                  class="rounded border border-dashed border-border/40 px-3 py-4 text-center text-xs text-muted-foreground"
                >
                  暂无组件
                </div>
              {:else}
                <div class="flex flex-col gap-2">
                  {#each row.zones[zoneId] as componentId (componentId)}
                    {@const componentIndex =
                      row.zones[zoneId].indexOf(componentId)}
                    <div
                      class="rounded-md border border-border/60 bg-background/70 px-3 py-2 text-xs text-foreground shadow-sm"
                    >
                      <div class="mb-2 font-medium">
                        {HEADER_LAYOUT_COMPONENT_LABELS[componentId]}
                      </div>
                      <div class="mb-2 flex flex-wrap gap-1.5">
                        {#each HEADER_LAYOUT_ZONE_IDS as targetZoneId (targetZoneId)}
                          <button
                            type="button"
                            class="rounded border px-2 py-1 transition-colors {zoneId ===
                            targetZoneId
                              ? 'border-border bg-muted text-foreground'
                              : 'border-border/60 text-muted-foreground hover:bg-popover/70 hover:text-foreground'}"
                            onclick={() =>
                              moveComponentToZone(
                                rowIndex,
                                zoneId,
                                componentId,
                                targetZoneId,
                              )}
                          >
                            {HEADER_LAYOUT_ZONE_LABELS[targetZoneId]}
                          </button>
                        {/each}
                      </div>
                      <div class="flex flex-wrap gap-1.5">
                        <button
                          type="button"
                          class="rounded border border-border/60 px-2 py-1 text-muted-foreground hover:bg-popover/70 hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={componentIndex === 0}
                          onclick={() =>
                            moveComponentInZone(
                              rowIndex,
                              zoneId,
                              componentId,
                              -1,
                            )}
                        >
                          左移
                        </button>
                        <button
                          type="button"
                          class="rounded border border-border/60 px-2 py-1 text-muted-foreground hover:bg-popover/70 hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={componentIndex ===
                            row.zones[zoneId].length - 1}
                          onclick={() =>
                            moveComponentInZone(
                              rowIndex,
                              zoneId,
                              componentId,
                              1,
                            )}
                        >
                          右移
                        </button>
                        <button
                          type="button"
                          class="rounded border border-border/60 px-2 py-1 text-muted-foreground hover:bg-popover/70 hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={rowIndex === 0}
                          onclick={() =>
                            moveComponentToRow(
                              rowIndex,
                              zoneId,
                              componentId,
                              rowIndex - 1,
                            )}
                        >
                          上一行
                        </button>
                        <button
                          type="button"
                          class="rounded border border-border/60 px-2 py-1 text-muted-foreground hover:bg-popover/70 hover:text-foreground disabled:cursor-not-allowed disabled:opacity-40"
                          disabled={rowIndex ===
                            normalizedLayout.rows.length - 1}
                          onclick={() =>
                            moveComponentToRow(
                              rowIndex,
                              zoneId,
                              componentId,
                              rowIndex + 1,
                            )}
                        >
                          下一行
                        </button>
                        <button
                          type="button"
                          class="rounded border border-border/60 px-2 py-1 text-muted-foreground hover:bg-popover/70 hover:text-foreground"
                          onclick={() =>
                            moveComponentToNewRow(
                              rowIndex,
                              zoneId,
                              componentId,
                            )}
                        >
                          拆到新行
                        </button>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</div>
