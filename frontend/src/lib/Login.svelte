<script>
    import IoIosArrowRoundForward from 'svelte-icons/io/IoIosArrowRoundForward.svelte';
    import { login } from './api';
</script>

<main class="w-full h-full flex flex-col justify-center items-center">
    <h1 class="text-4xl">Journal</h1>

    <button class="history opacity-50 hover:opacity-100 transition-opacity" on:click={login}>
        Login
        <span class="ml-1 mt-px w-4 h-4 inline-block"><IoIosArrowRoundForward /></span>
    </button>
</main>

<style lang="postcss">
    button {
        @apply flex items-center mx-auto px-4;
        font-size: 0.75rem;
    }
</style>
