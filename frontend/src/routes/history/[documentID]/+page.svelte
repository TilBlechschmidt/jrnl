<script lang="ts">
    import { nth } from '$lib';

    export let data;

    const { identifier, contents } = data;
    const date = new Date(identifier);
</script>

<article class="w-full flex flex-col items-center pt-16 pb-16">
    <h1 class="text-center pb-8">
        <span class="italic">
            <span class="text-sm opacity-50 pl-1">The</span>
            {date.getDate() + nth(date.getDate())}
            <span class="text-sm opacity-50 pr-1">of</span>
        </span>
        <span class="text-4xl block">
            {date.toLocaleString('default', { month: 'long' })}
            {date.getFullYear()}
        </span>
        <span class="italic">
            <span class="text-sm opacity-50">~</span>
            {date.getHours()}:{date.getMinutes()}
            <span class="text-sm opacity-50">~</span>
        </span>
    </h1>
    <section class="w-full max-w-lg opacity-85 whitespace-pre-wrap">
        {contents}
    </section>
</article>
