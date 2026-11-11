<script lang="ts">
  import { t } from "../../i18n";

  interface Props {
    name: string;
    color: string;
  }

  let { name, color }: Props = $props();
</script>

<span class="chip" style:--tag={color} title={t.tags.tagOf(name)}>{name}</span>

<style>
  .chip {
    display: inline-block;
    flex: none;
    max-width: 110px;
    padding: 0 7px;
    border-radius: var(--radius-chip);
    background: color-mix(in srgb, var(--tag) 20%, transparent);
    color: color-mix(in srgb, var(--tag) 55%, var(--text-primary));
    font-size: 11px;
    font-weight: 600;
    line-height: 18px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
