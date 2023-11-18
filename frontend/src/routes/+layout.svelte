<script lang="ts">
    import '../app.css';

    import Login from '$lib/Login.svelte';
    import { user } from '$lib';
</script>

{#if user}
    <slot />
{:else}
    <Login />
{/if}
