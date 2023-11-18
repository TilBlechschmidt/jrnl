<script lang="ts">
    import Title from './Title.svelte';
    import type { Document } from '$lib/api';
    import { nth } from '$lib';

    export let documents: Document[];

    interface Group {
        title: string;
        docs: Document[];
    }

    function groupByMonth(documents: Document[]): Group[] {
        if (documents.length == 0) return [];

        const initial = new Date(documents[0].identifier);
        let month = initial.getMonth();
        let year = initial.getFullYear();
        let pending: Document[] = [];

        const groups = [];

        for (let doc of documents) {
            const date = new Date(doc.identifier);

            if (date.getMonth() != month || date.getFullYear() != year) {
                groups.push({
                    title: groupTitle(pending),
                    docs: pending
                });

                pending = [];
                month = date.getMonth();
                year = date.getFullYear();
            }

            pending.push(doc);
        }

        groups.push({
            title: groupTitle(pending),
            docs: pending
        });

        return groups;
    }

    function groupTitle(documents: Document[]): string {
        const date = new Date(documents[0].identifier);
        const humanReadableMonth = date.toLocaleString('default', {
            month: 'long'
        });

        return `${humanReadableMonth} ${date.getFullYear()}`;
    }

    $: groups = groupByMonth(documents);
</script>

<main class="w-full flex flex-col items-center pt-16">
    <Title />
    {#each groups as group}
        <section class="w-full max-w-xs">
            <h2 class="text-lg pb-4 pt-8 text-center">
                <span class="opacity-50 pr-2">~</span>
                {group.title}
                <span class="opacity-50 pl-2">~</span>
            </h2>
            {#each group.docs as doc}
                {@const date = new Date(doc.identifier).getDate()}
                <a class="relative text-sm text-left" href={`/history/${doc.identifier}`}>
                    <h3 class="date">{date + nth(date)}</h3>
                    <span
                        class="line-clamp overflow-hidden opacity-75 hover:opacity-100 transition-opacity duration-500"
                    >
                        {doc.contents}
                    </span>
                    {#if true}<hr class="opacity-10 my-4 mx-4" />{/if}
                </a>
            {/each}
        </section>
    {/each}

    <div class="w-full"><br /><br /><br /><br /></div>
</main>

<style lang="postcss">
    .date {
        @apply absolute top-0 -left-16 w-16 text-right pr-4 opacity-50;
        font-size: 0.75rem;
    }

    .line-clamp {
        display: -webkit-box;
        -webkit-line-clamp: 3;
        -webkit-box-orient: vertical;
    }
</style>
