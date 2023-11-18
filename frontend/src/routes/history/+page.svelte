<script>
    import History from '$lib/History.svelte';

    export let data;
</script>

<History documents={data.documents} />
