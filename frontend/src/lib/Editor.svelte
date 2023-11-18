<script lang="ts">
    import IoIosArrowRoundForward from 'svelte-icons/io/IoIosArrowRoundForward.svelte';
    import { documentIdentifier, documentContent } from './stores';
    import { goto } from '$app/navigation';
    import * as api from '$lib/api';
    import Title from './Title.svelte';

    const saveInterval = 500;
    const saveKeystrokes = 128;
    const prefix = '\n'.repeat(5);

    let textarea: HTMLTextAreaElement;

    let keystrokes = 0;
    let saveTimeout: number | null = null;

    function setPosition() {
        // Make sure there is always 5 leading newlines
        if (!$documentContent.startsWith(prefix)) {
            $documentContent = prefix + $documentContent.trimStart();
        }

        // Pin the cursor to the end
        textarea.selectionStart = $documentContent.length;
        textarea.selectionEnd = $documentContent.length;
    }

    function handleKeyEvent(e: KeyboardEvent) {
        const isArrow = e.code.startsWith('Arrow');
        const isFind = e.code == 'KeyF' && (e.ctrlKey || e.metaKey);
        const isSave = e.code == 'KeyS' && (e.ctrlKey || e.metaKey);

        if (isArrow || isFind || isSave) {
            e.preventDefault();
            return false;
        } else {
            setPosition();
        }

        if (keystrokes++ > saveKeystrokes) {
            saveDocument();
        } else {
            if (saveTimeout) clearTimeout(saveTimeout);
            saveTimeout = setTimeout(saveDocument, saveInterval);
        }
    }

    $: trimmed = $documentContent.trim();
    $: wordCount = trimmed.length > 0 ? trimmed.split(/\s+/).length : 0;
    $: empty = !trimmed;

    async function saveDocument() {
        keystrokes = 0;

        if (!empty) {
            // TODO Don't run this twice in parallel!
            await api.write($documentIdentifier, $documentContent.trim());
        }
    }
</script>

<main class="editor">
    <div class="content">
        {#if empty}
            <span class="placeholder"> Start a new entry ... </span>
        {/if}

        <!-- svelte-ignore a11y-autofocus -->
        <textarea
            autofocus
            bind:value={$documentContent}
            bind:this={textarea}
            on:select={setPosition}
            on:click={setPosition}
            on:focus={setPosition}
            on:paste={setPosition}
            on:keydown={handleKeyEvent}
            on:keyup={handleKeyEvent}
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
        />

        <button class="overlay" on:click={() => textarea.focus()}>
            <span />
            <span />
            <span />
            <span />
            <span />
        </button>

        <div
            class="absolute top-0 left-0 w-full pointer-events-none transition-opacity duration-1000"
            class:opacity-85={empty}
            class:opacity-0={!empty}
        >
            <Title />
        </div>
    </div>

    <div
        class="absolute bottom-8 left-0 w-full transition-opacity duration-500"
        class:opacity-100={empty}
        class:opacity-0={!empty}
    >
        <button
            class="history opacity-50 hover:opacity-100 transition-opacity"
            on:click={() => goto('/history')}
        >
            View previous entries
            <span class="ml-1 mt-px w-4 h-4 inline-block"><IoIosArrowRoundForward /></span>
        </button>
    </div>

    <div class="absolute bottom-8 right-12 opacity-20">
        {wordCount}
    </div>
</main>

<style lang="postcss">
    .history {
        @apply flex items-center mx-auto px-4;
        font-size: 0.75rem;
    }

    .editor {
        @apply w-full h-full flex flex-col justify-center items-center;

        font-size: 1rem;
        line-height: calc(var(--ru) * 1rem);
    }

    .content {
        @apply w-full relative;

        height: calc(var(--ru) * 6rem);
        top: calc(var(--ru) * -2.5rem);
        max-width: 25rem;
    }

    textarea {
        @apply absolute w-full h-full overflow-hidden border-none outline-none resize-none bg-transparent;

        padding: 0 calc(var(--ru) * 1rem);
    }

    textarea::selection {
        background-color: transparent;
    }

    .overlay {
        @apply absolute flex flex-col items-start top-0 left-0 right-0 cursor-text;
        height: calc(var(--ru) * 5rem);
    }

    .overlay span {
        @apply w-full;

        background-color: var(--page-color);
        height: calc(var(--ru) * 1rem);
    }

    .overlay span:first-child {
        opacity: 0.98;
    }

    .overlay span:nth-child(2) {
        opacity: 0.92;
    }

    .overlay span:nth-child(3) {
        opacity: 0.85;
    }

    .overlay span:nth-child(4) {
        opacity: 0.7;
    }

    .overlay span:nth-child(5) {
        opacity: 0.5;
    }

    .placeholder {
        @apply bg-transparent absolute bottom-0 opacity-25;
        padding: 0 calc(var(--ru) * 1rem);
    }
</style>
